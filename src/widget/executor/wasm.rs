use crate::drawer::{
    CircleData, Color, Drawer, LineData, Point, Rect, RectData, Size, TextData, TriangleData,
};
use crate::widget::executor::{Context, Executor};
use core::ptr::NonNull;
use wasmi::{Caller, Config, Engine, Extern, Linker, Module, Store, TypedFunc};

trait HostModule {
    fn name(&self) -> &str;
    fn register(
        &self,
        linker: &mut Linker<WasmCtx>,
        store: &mut Store<WasmCtx>,
    ) -> Result<(), wasmi::Error>;
}

struct WasmCtx {
    drawer: Option<NonNull<dyn Drawer>>,
}

impl WasmCtx {
    fn new() -> Self {
        Self { drawer: None }
    }

    fn drawer(&self) -> &mut dyn Drawer {
        unsafe { self.drawer.unwrap().as_mut() }
    }
}

struct DrawerModule;

impl HostModule for DrawerModule {
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
            "draw_rect",
            wasmi::Func::wrap(
                &mut *store,
                |mut caller: Caller<'_, WasmCtx>,
                 x: u32,
                 y: u32,
                 w: u32,
                 h: u32,
                 r: u32,
                 g: u32,
                 b: u32,
                 fill: u32,
                 stroke_width: u32,
                 corner_radius: u32| {
                    let data = RectData {
                        rect: Rect {
                            origin: Point { x, y },
                            size: Size {
                                width: w,
                                height: h,
                            },
                        },
                        color: Color::Rgb(r as u8, g as u8, b as u8),
                        fill: fill != 0,
                        stroke_width,
                        corner_radius,
                    };
                    caller.data_mut().drawer().execute_rect(data);
                },
            ),
        )?;

        linker.define(
            self.name(),
            "draw_circle",
            wasmi::Func::wrap(
                &mut *store,
                |mut caller: Caller<'_, WasmCtx>,
                 cx: u32,
                 cy: u32,
                 radius: u32,
                 r: u32,
                 g: u32,
                 b: u32,
                 fill: u32,
                 stroke_width: u32| {
                    let data = CircleData {
                        center: Point { x: cx, y: cy },
                        radius,
                        color: Color::Rgb(r as u8, g as u8, b as u8),
                        fill: fill != 0,
                        stroke_width,
                    };
                    caller.data_mut().drawer().execute_circle(data);
                },
            ),
        )?;

        linker.define(
            self.name(),
            "draw_line",
            wasmi::Func::wrap(
                &mut *store,
                |mut caller: Caller<'_, WasmCtx>,
                 x1: u32,
                 y1: u32,
                 x2: u32,
                 y2: u32,
                 r: u32,
                 g: u32,
                 b: u32,
                 thickness: u32| {
                    let data = LineData {
                        start: Point { x: x1, y: y1 },
                        end: Point { x: x2, y: y2 },
                        color: Color::Rgb(r as u8, g as u8, b as u8),
                        thickness,
                    };
                    caller.data_mut().drawer().execute_line(data);
                },
            ),
        )?;

        linker.define(
            self.name(),
            "draw_triangle",
            wasmi::Func::wrap(
                &mut *store,
                |mut caller: Caller<'_, WasmCtx>,
                 x1: u32,
                 y1: u32,
                 x2: u32,
                 y2: u32,
                 x3: u32,
                 y3: u32,
                 r: u32,
                 g: u32,
                 b: u32,
                 fill: u32,
                 stroke_width: u32| {
                    let data = TriangleData {
                        p1: Point { x: x1, y: y1 },
                        p2: Point { x: x2, y: y2 },
                        p3: Point { x: x3, y: y3 },
                        color: Color::Rgb(r as u8, g as u8, b as u8),
                        fill: fill != 0,
                        stroke_width,
                    };
                    caller.data_mut().drawer().execute_triangle(data);
                },
            ),
        )?;

        linker.define(
            self.name(),
            "draw_text",
            wasmi::Func::wrap(
                &mut *store,
                |mut caller: Caller<'_, WasmCtx>,
                 ptr: u32,
                 len: u32,
                 x: u32,
                 y: u32,
                 r: u32,
                 g: u32,
                 b: u32| {
                    let memory = match caller.get_export("memory") {
                        Some(Extern::Memory(m)) => m,
                        _ => return,
                    };

                    let start = ptr as usize;
                    let end = start + len as usize;
                    let memory_data = memory.data(&caller);
                    if end > memory_data.len() {
                        return;
                    }

                    let bytes = match memory_data.get(start..end) {
                        Some(bytes) => bytes,
                        None => return,
                    };
                    let text = match core::str::from_utf8(bytes) {
                        Ok(text) => text,
                        Err(_) => return,
                    };
                    let mut buf = heapless::String::<256>::new();
                    if let Err(_e) = buf.push_str(text) {
                        return;
                    }

                    let data = TextData {
                        text: &*buf,
                        position: Point { x, y },
                        color: Color::Rgb(r as u8, g as u8, b as u8),
                    };
                    caller.data_mut().drawer().execute_text(data);
                },
            ),
        )?;

        linker.define(
            self.name(),
            "clear",
            wasmi::Func::wrap(
                &mut *store,
                |mut caller: Caller<'_, WasmCtx>, r: u32, g: u32, b: u32| {
                    caller
                        .data_mut()
                        .drawer()
                        .execute_clear(Color::Rgb(r as u8, g as u8, b as u8));
                },
            ),
        )?;

        Ok(())
    }
}

struct SystemModule;

impl HostModule for SystemModule {
    fn name(&self) -> &str {
        "system"
    }

    fn register(
        &self,
        linker: &mut Linker<WasmCtx>,
        store: &mut Store<WasmCtx>,
    ) -> Result<(), wasmi::Error> {
        linker.define(
            self.name(),
            "get_time",
            wasmi::Func::wrap(store, |_caller: Caller<'_, WasmCtx>| {
                // TODO
                0
            }),
        )?;

        Ok(())
    }
}

pub struct WasmExecutor {
    render_func: TypedFunc<(), ()>,
    store: Store<WasmCtx>,
}

impl WasmExecutor {
    pub fn new(wasm_binary: &[u8]) -> Result<Self, wasmi::Error> {
        Self::with_modules(
            wasm_binary,
            alloc::vec![
                alloc::boxed::Box::new(DrawerModule),
                alloc::boxed::Box::new(SystemModule)
            ],
        )
    }

    fn with_modules(
        wasm_binary: &[u8],
        host_modules: alloc::vec::Vec<alloc::boxed::Box<dyn HostModule>>,
    ) -> Result<Self, wasmi::Error> {
        let mut config = Config::default();
        config.consume_fuel(false);
        let engine = Engine::new(&config);
        let module = Module::new(&engine, wasm_binary)?;

        let mut linker = Linker::new(&engine);
        let ctx = WasmCtx::new();
        let mut store = Store::new(&engine, ctx);
        for module in host_modules {
            module.register(&mut linker, &mut store)?;
        }

        let instance = linker.instantiate_and_start(&mut store, &module)?;
        let render_func = instance.get_typed_func::<(), ()>(&store, "render")?;

        Ok(Self { render_func, store })
    }
}

impl Executor for WasmExecutor {
    fn render(&mut self, ctx: Context) {
        let drawer = ctx.drawer as *mut dyn Drawer;

        // SAFETY:
        // `ctx.drawer` is valid for the duration of this render() call.
        // We temporarily erase its lifetime because wasmi host functions access store data
        // synchronously while the Wasm render function is executing.
        // The pointer must be cleared before render() returns.
        let drawer: *mut (dyn Drawer + 'static) = unsafe { core::mem::transmute(drawer) };

        self.store.data_mut().drawer = NonNull::new(drawer);

        self.render_func.call(&mut self.store, ()).unwrap();

        self.store.data_mut().drawer = None;
    }
}
