use crate::drawer::Drawer;
use crate::widget::executor::{Context, Executor};
use core::ptr::NonNull;
use wasmi::{Caller, Config, Engine, Linker, Module, Store, StoreLimits, StoreLimitsBuilder, TypedFunc};

const MAX_WASM_MEMORY_BYTES: usize = 10240;

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
    limits: StoreLimits,
}

impl WasmCtx {
    fn new() -> Self {
        Self {
            drawer: None,
            limits: StoreLimitsBuilder::new()
                .memory_size(MAX_WASM_MEMORY_BYTES)
                .memories(1)
                .instances(1)
                .build(),
        }
    }

    fn drawer(&self) -> &mut dyn Drawer {
        unsafe { self.drawer.unwrap().as_mut() }
    }
}

struct DrawerModule;

include!(concat!(env!("OUT_DIR"), "/drawer_wasm_bindings.rs"));

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
                 _col: i32| {

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
        )
    }

    fn with_modules(
        wasm_binary: &[u8],
    ) -> Result<Self, wasmi::Error> {
        let mut config = Config::default();
        config.consume_fuel(false);
        let engine = Engine::new(&config);
        let module = Module::new(&engine, wasm_binary)?;

        let mut linker = Linker::new(&engine);
        let ctx = WasmCtx::new();
        let mut store = Store::new(&engine, ctx);
        store.limiter(|ctx| &mut ctx.limits);

        DrawerModule.register(&mut linker, &mut store)?;
        SystemModule.register(&mut linker, &mut store)?;
        EnvModule.register(&mut linker, &mut store)?;

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
