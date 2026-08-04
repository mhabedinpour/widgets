use crate::storage::FS;
use embedded_io_async::{Read, Write};
use littlefs2::io::OpenSeekFrom;
use littlefs2::path::PathBuf;
use log::error;
use picoserve::ResponseSent;
use picoserve::response::{
    Connection, Content, IntoResponse, Response, ResponseWriter, StatusCode,
};

const CHUNK_SIZE: usize = 512;

pub struct FileBody {
    path: PathBuf,
    first_chunk: heapless::Vec<u8, CHUNK_SIZE>,
    file_len: usize,
    fs: FS,
}

impl Content for FileBody {
    fn content_type(&self) -> &'static str {
        let path_str = self.path.as_str();
        if path_str.ends_with(".html") {
            "text/html; charset=utf-8"
        } else if path_str.ends_with(".css") {
            "text/css"
        } else if path_str.ends_with(".js") {
            "application/javascript; charset=utf-8"
        } else if path_str.ends_with(".json") {
            "application/json"
        } else {
            "application/octet-stream"
        }
    }

    fn content_length(&self) -> usize {
        self.file_len
    }

    // Write file body contents
    async fn write_content<W: Write>(self, mut writer: W) -> Result<(), W::Error> {
        let mut pos = 0;

        if !self.first_chunk.is_empty() {
            writer.write_all(&self.first_chunk).await?;
            pos += self.first_chunk.len();
        }

        while pos < self.file_len {
            match self
                .fs
                .read_chunk::<CHUNK_SIZE>(&self.path, OpenSeekFrom::Start(pos as u32))
            {
                Ok((chunk, _)) => {
                    if chunk.is_empty() {
                        break; // EOF
                    }
                    writer.write_all(&chunk).await?;
                    pos += chunk.len();
                }
                Err(e) => {
                    error!("Read error at pos {pos} for {:?}: {e:?}", self.path);
                    break;
                }
            }
        }

        Ok(())
    }
}

pub struct FileResponse {
    path: &'static str,
    fs: FS,
}

impl FileResponse {
    pub async fn from(fs: FS, path: &'static str) -> Self {
        Self { fs, path }
    }
}

impl IntoResponse for FileResponse {
    async fn write_to<R: Read, W: ResponseWriter<Error = R::Error>>(
        self,
        connection: Connection<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        let path_buf = match PathBuf::try_from(self.path) {
            Ok(p) => p,
            Err(_) => {
                return StatusCode::BAD_REQUEST
                    .write_to(connection, response_writer)
                    .await;
            }
        };

        // Check if file exists by reading the 1st chunk
        let (first_chunk, file_len) = match self
            .fs
            .read_chunk::<CHUNK_SIZE>(&path_buf, OpenSeekFrom::Start(0))
        {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to open file {:?}: {:?}", self.path, e);
                return StatusCode::NOT_FOUND
                    .write_to(connection, response_writer)
                    .await;
            }
        };

        let file_body = FileBody {
            path: path_buf,
            first_chunk,
            file_len,
            fs: self.fs,
        };
        let response = Response::ok(file_body);
        response_writer.write_response(connection, response).await
    }
}
