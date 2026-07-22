use crate::console::LogData;
use crate::drawer::{Baseline, ClearData, Color, Font, Point, TextAlignment, TextData};
use crate::widget::WidgetEvent;
use crate::widget::executor::{Context, Executor};
use crate::{use_psram_heap, use_sram_heap};
use alloc::string::String;
use alloc::vec::Vec;
use wasmi::{
    Caller, Config, Engine, Linker, Module, Store, StoreLimits, StoreLimitsBuilder, TypedFunc,
};

const MAX_WASM_MEMORY_BYTES: usize = 100 * 1024;

trait HostModule {
    fn name(&self) -> &str;
    fn register(
        &self,
        linker: &mut Linker<WasmCtx>,
        store: &mut Store<WasmCtx>,
    ) -> Result<(), wasmi::Error>;
}

struct WasmCtx {
    ctx: Option<Context>,
    limits: StoreLimits,
    event_queue: Vec<WidgetEvent>,
    event_cursor: Option<usize>,
}

impl WasmCtx {
    fn new() -> Self {
        Self {
            ctx: None,
            limits: StoreLimitsBuilder::new()
                .memory_size(MAX_WASM_MEMORY_BYTES)
                .build(),
            event_queue: Vec::new(),
            event_cursor: None,
        }
    }
}

struct DrawerModule;

include!(concat!(env!("OUT_DIR"), "/drawer_wasm_bindings.rs"));

struct TimeModule;

include!(concat!(env!("OUT_DIR"), "/time_wasm_bindings.rs"));

struct EventModule;

include!(concat!(env!("OUT_DIR"), "/event_wasm_bindings.rs"));

struct HttpModule;

include!(concat!(env!("OUT_DIR"), "/http_wasm_bindings.rs"));

struct ConsoleModule;

include!(concat!(env!("OUT_DIR"), "/console_wasm_bindings.rs"));

struct NetworkModule;

include!(concat!(env!("OUT_DIR"), "/network_wasm_bindings.rs"));

struct ConfigModule;

impl HostModule for ConfigModule {
    fn name(&self) -> &str {
        "config"
    }

    fn register(
        &self,
        linker: &mut Linker<WasmCtx>,
        store: &mut Store<WasmCtx>,
    ) -> Result<(), wasmi::Error> {
        linker.define(
            self.name(),
            "get",
            wasmi::Func::wrap(
                &mut *store,
                |mut caller: Caller<'_, WasmCtx>,
                 key_len: u32,
                 key_ptr: u32,
                 out_max: u32,
                 out_ptr: u32|
                 -> Result<i32, wasmi::Error> {
                    let memory = match caller.get_export("memory") {
                        Some(wasmi::Extern::Memory(m)) => m,
                        _ => return Err(wasmi::Error::new("guest memory export missing")),
                    };

                    // Read the key into an owned String so we can release the
                    // immutable borrow before writing back to memory.
                    let key: String = {
                        let start = key_ptr as usize;
                        let end = start
                            .checked_add(key_len as usize)
                            .ok_or_else(|| wasmi::Error::new("key pointer overflow"))?;
                        let bytes = memory
                            .data(&caller)
                            .get(start..end)
                            .ok_or_else(|| wasmi::Error::new("out-of-bounds key pointer"))?;
                        let s = core::str::from_utf8(bytes)
                            .map_err(|_| wasmi::Error::new("key is not valid UTF-8"))?;
                        String::from(s)
                    };

                    // Look up the value and copy bytes; releases the data borrow.
                    let value_bytes: Option<Vec<u8>> = {
                        let result = caller
                            .data()
                            .ctx
                            .as_ref()
                            .unwrap()
                            .config
                            .get(key.as_str())
                            .map(|v| v.as_bytes().to_vec());
                        result
                    };

                    match value_bytes {
                        None => Ok(-1i32),
                        Some(bytes) => {
                            let write_len = bytes.len().min(out_max as usize);
                            let start = out_ptr as usize;
                            let end = start
                                .checked_add(write_len)
                                .ok_or_else(|| wasmi::Error::new("output pointer overflow"))?;
                            memory
                                .data_mut(&mut caller)
                                .get_mut(start..end)
                                .ok_or_else(|| wasmi::Error::new("out-of-bounds output pointer"))?
                                .copy_from_slice(&bytes[..write_len]);
                            Ok(write_len as i32)
                        }
                    }
                },
            ),
        )?;
        Ok(())
    }
}

struct EnvModule;

impl HostModule for EnvModule {
    fn name(&self) -> &str {
        "env"
    }

    fn register(
        &self,
        linker: &mut Linker<WasmCtx>,
        store: &mut Store<WasmCtx>,
    ) -> Result<(), wasmi::Error> {
        linker.define(
            self.name(),
            "abort",
            wasmi::Func::wrap(
                store,
                |_caller: Caller<'_, WasmCtx>,
                 _message: i32,
                 _file_name: i32,
                 _line: i32,
                 _col: i32| {},
            ),
        )?;

        Ok(())
    }
}

pub struct WasmExecutor {
    render_func: Option<TypedFunc<(), ()>>,
    store: Store<WasmCtx>,
    failed: bool,
    init_error: Option<String>,
}

impl WasmExecutor {
    /// Instantiate the widget module. Never fails: on error the executor
    /// logs the cause on first render and paints an error state instead.
    pub fn new(wasm_binary: &[u8]) -> Self {
        Self::with_modules(wasm_binary).unwrap_or_else(|err| {
            use_sram_heap();

            let engine = Engine::new(&Config::default());
            let store = Store::new(&engine, WasmCtx::new());
            Self {
                render_func: None,
                store,
                failed: true,
                init_error: Some(alloc::format!("widget init failed: {}", err)),
            }
        })
    }

    fn with_modules(wasm_binary: &[u8]) -> Result<Self, wasmi::Error> {
        use_psram_heap();

        let mut config = Config::default();
        config.consume_fuel(false);
        config.set_min_stack_height(512);
        config.set_max_stack_height(4 * 1024);
        config.set_max_cached_stacks(0);

        let engine = Engine::new(&config);
        let module = Module::new(&engine, wasm_binary)?;

        let mut linker = Linker::new(&engine);
        let ctx = WasmCtx::new();
        let mut store = Store::new(&engine, ctx);
        store.limiter(|ctx| &mut ctx.limits);

        DrawerModule.register(&mut linker, &mut store)?;
        TimeModule.register(&mut linker, &mut store)?;
        EnvModule.register(&mut linker, &mut store)?;
        EventModule.register(&mut linker, &mut store)?;
        HttpModule.register(&mut linker, &mut store)?;
        ConsoleModule.register(&mut linker, &mut store)?;
        NetworkModule.register(&mut linker, &mut store)?;
        ConfigModule.register(&mut linker, &mut store)?;

        let instance = linker.instantiate_and_start(&mut store, &module)?;
        let render_func = instance.get_typed_func::<(), ()>(&store, "render")?;

        use_sram_heap();

        Ok(Self {
            render_func: Some(render_func),
            store,
            failed: false,
            init_error: None,
        })
    }

    /// Paint an error indicator across the widget's bounds.
    fn draw_error(&mut self) {
        if let Some(ctx) = self.store.data_mut().ctx.as_mut() {
            let drawer = &mut ctx.drawer;
            let w = drawer.bounds_width();
            let h = drawer.bounds_height();

            drawer.execute_clear(ClearData {
                color: Color::BLACK,
            });

            // Pick a font that fits the band height.
            let font = if h >= 14 {
                Font::Font5x8
            } else {
                Font::U8g2Font3x5
            };

            drawer.execute_text(TextData {
                text: String::from("ERROR"),
                position: Point::new(w / 2, h / 2),
                color: Color::Rgb(255, 60, 50),
                background_color: Color::BLACK,
                has_background: false,
                font,
                underline: false,
                strikethrough: false,
                alignment: TextAlignment::Center,
                baseline: Baseline::Middle,
            });
        }

        use_sram_heap();
    }
}

impl Executor for WasmExecutor {
    fn set_ctx(&mut self, ctx: Context) {
        self.store.data_mut().ctx = Some(ctx);
    }

    fn render(&mut self, events: Option<Vec<WidgetEvent>>) {
        use_psram_heap();

        if let Some(message) = self.init_error.take() {
            if let Some(ctx) = self.store.data_mut().ctx.as_mut() {
                ctx.console.log_error(LogData { message });
            }
        }

        if self.failed {
            // Init failed or the instance already trapped; keep showing the
            // error instead of calling into a missing/corrupt module.
            self.draw_error();
            return;
        }

        let data = self.store.data_mut();
        data.event_queue = events.unwrap_or_default();
        data.event_cursor = None;

        let render_func = match self.render_func {
            Some(f) => f,
            None => {
                self.failed = true;
                self.draw_error();
                return;
            }
        };

        if let Err(err) = render_func.call(&mut self.store, ()) {
            self.failed = true;
            let message = alloc::format!("widget render failed: {}", err);
            if let Some(ctx) = self.store.data_mut().ctx.as_mut() {
                ctx.console.log_error(LogData { message });
            }
            self.draw_error();
        }

        use_sram_heap();
    }
}
