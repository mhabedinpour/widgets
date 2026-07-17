use crate::widget::WidgetEvent;
use crate::widget::executor::{Context, Executor};
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
}

impl WasmCtx {
    fn new() -> Self {
        Self {
            ctx: None,
            limits: StoreLimitsBuilder::new()
                .memory_size(MAX_WASM_MEMORY_BYTES)
                .build(),
        }
    }
}

struct DrawerModule;

include!(concat!(env!("OUT_DIR"), "/drawer_wasm_bindings.rs"));

struct TimerModule;

include!(concat!(env!("OUT_DIR"), "/timer_wasm_bindings.rs"));

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
    render_func: TypedFunc<(), ()>,
    store: Store<WasmCtx>,
}

impl WasmExecutor {
    pub fn new(wasm_binary: &[u8]) -> Result<Self, wasmi::Error> {
        Self::with_modules(wasm_binary)
    }

    fn with_modules(wasm_binary: &[u8]) -> Result<Self, wasmi::Error> {
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
        TimerModule.register(&mut linker, &mut store)?;
        EnvModule.register(&mut linker, &mut store)?;

        let instance = linker.instantiate_and_start(&mut store, &module)?;
        let render_func = instance.get_typed_func::<(), ()>(&store, "render")?;

        Ok(Self { render_func, store })
    }
}

impl Executor for WasmExecutor {
    fn set_ctx(&mut self, ctx: Context) {
        self.store.data_mut().ctx = Some(ctx);
    }

    fn render(&mut self, events: Option<Vec<WidgetEvent>>) {
        self.render_func.call(&mut self.store, ()).unwrap();
    }
}
