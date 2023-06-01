use std::collections::HashMap;

use cosmwasm_std::{coins, Empty};
use cosmwasm_vm::internals::make_engine;
use cosmwasm_vm::testing::{mock_backend, mock_env, mock_info};

use cosmwasm_vm::{call_instantiate, Instance};

use wasmer::{Engine, Module, Store};

static CONTRACT: &[u8] = include_bytes!("../testdata/empty.wasm");

fn call_instantiate_on_cached_contract() {
    let mut cache = HashMap::new();

    let (_engine, module) = compile(CONTRACT);

    let engine = Engine::headless();
    cache.insert("asdf", (engine, module));

    {
        let (cached_engine, cached_module) = cache.get("asdf").unwrap();
        let store = Store::new(cached_engine.clone());
        let mut instance = Instance::from_module(
            store,
            cached_module,
            mock_backend(&[]),
            u64::MAX,
            false,
            None,
            None,
        )
        .unwrap();

        // init
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = br#"{}"#;
        let res =
            call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(msgs.len(), 0);
    }
}

fn compile(code: &[u8]) -> (Engine, Module) {
    let engine = make_engine(&[]);
    let module = Module::new(&engine, code).unwrap();
    (engine, module)
}

pub fn main() {
    // run this with `cargo run --example test_multithreaded --release` for faster failure
    loop {
        run_threads(call_instantiate_on_cached_contract);
    }
}

fn run_threads(function: fn() -> ()) {
    // spawn many threads, even if there are few functions
    const TARGET_THREAD_COUNT: usize = 20;
    let mut threads = Vec::with_capacity(TARGET_THREAD_COUNT);

    for _ in 0..TARGET_THREAD_COUNT {
        let thread = std::thread::spawn(function);
        threads.push(thread);
    }

    for thread in threads {
        thread.join().unwrap();
    }
}
