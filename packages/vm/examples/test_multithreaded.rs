use std::collections::HashSet;

use cosmwasm_std::{coins, Empty};
use cosmwasm_vm::testing::{mock_backend, mock_env, mock_info};

use cosmwasm_vm::Size;
use cosmwasm_vm::{call_execute, call_instantiate};
use cosmwasm_vm::{capabilities_from_csv, CacheOptions};

use cosmwasm_vm::Cache;
use cosmwasm_vm::InstanceOptions;

use tempfile::TempDir;

const TESTING_GAS_LIMIT: u64 = 500_000_000_000; // ~0.5ms
const TESTING_MEMORY_LIMIT: Size = Size::mebi(16);
const TESTING_OPTIONS: InstanceOptions = InstanceOptions {
    gas_limit: TESTING_GAS_LIMIT,
    print_debug: false,
};
const TESTING_MEMORY_CACHE_SIZE: Size = Size::mebi(200);

static CONTRACT: &[u8] = include_bytes!("../testdata/hackatom.wasm");

fn default_capabilities() -> HashSet<String> {
    capabilities_from_csv("iterator,staking")
}

fn make_testing_options() -> CacheOptions {
    CacheOptions {
        base_dir: TempDir::new().unwrap().into_path(),
        available_capabilities: default_capabilities(),
        memory_cache_size: TESTING_MEMORY_CACHE_SIZE,
        instance_memory_limit: TESTING_MEMORY_LIMIT,
    }
}

fn call_instantiate_on_cached_contract() {
    let cache = unsafe { Cache::new(make_testing_options()).unwrap() };
    let checksum = cache.save_wasm(CONTRACT).unwrap();

    // from file system
    {
        let mut instance = cache
            .get_instance(&checksum, mock_backend(&[]), TESTING_OPTIONS)
            .unwrap();
        assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
        assert_eq!(cache.stats().hits_memory_cache, 0);
        assert_eq!(cache.stats().hits_fs_cache, 1);
        assert_eq!(cache.stats().misses, 0);

        // init
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
        let res =
            call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(msgs.len(), 0);
    }

    // from memory
    {
        let mut instance = cache
            .get_instance(&checksum, mock_backend(&[]), TESTING_OPTIONS)
            .unwrap();
        assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
        assert_eq!(cache.stats().hits_memory_cache, 1);
        assert_eq!(cache.stats().hits_fs_cache, 1);
        assert_eq!(cache.stats().misses, 0);

        // init
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
        let res =
            call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(msgs.len(), 0);
    }

    // from pinned memory
    {
        cache.pin(&checksum).unwrap();

        let mut instance = cache
            .get_instance(&checksum, mock_backend(&[]), TESTING_OPTIONS)
            .unwrap();
        assert_eq!(cache.stats().hits_pinned_memory_cache, 1);
        assert_eq!(cache.stats().hits_memory_cache, 1);
        assert_eq!(cache.stats().hits_fs_cache, 2);
        assert_eq!(cache.stats().misses, 0);

        // init
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
        let res =
            call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(msgs.len(), 0);
    }
}

fn call_execute_on_cached_contract() {
    let cache = unsafe { Cache::new(make_testing_options()).unwrap() };
    let checksum = cache.save_wasm(CONTRACT).unwrap();

    // from file system
    {
        let mut instance = cache
            .get_instance(&checksum, mock_backend(&[]), TESTING_OPTIONS)
            .unwrap();
        assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
        assert_eq!(cache.stats().hits_memory_cache, 0);
        assert_eq!(cache.stats().hits_fs_cache, 1);
        assert_eq!(cache.stats().misses, 0);

        // init
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
        let response = call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();
        assert_eq!(response.messages.len(), 0);

        // execute
        let info = mock_info("verifies", &coins(15, "earth"));
        let msg = br#"{"release":{}}"#;
        let response = call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();
        assert_eq!(response.messages.len(), 1);
    }

    // from memory
    {
        let mut instance = cache
            .get_instance(&checksum, mock_backend(&[]), TESTING_OPTIONS)
            .unwrap();
        assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
        assert_eq!(cache.stats().hits_memory_cache, 1);
        assert_eq!(cache.stats().hits_fs_cache, 1);
        assert_eq!(cache.stats().misses, 0);

        // init
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
        let response = call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();
        assert_eq!(response.messages.len(), 0);

        // execute
        let info = mock_info("verifies", &coins(15, "earth"));
        let msg = br#"{"release":{}}"#;
        let response = call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();
        assert_eq!(response.messages.len(), 1);
    }

    // from pinned memory
    {
        cache.pin(&checksum).unwrap();

        let mut instance = cache
            .get_instance(&checksum, mock_backend(&[]), TESTING_OPTIONS)
            .unwrap();
        assert_eq!(cache.stats().hits_pinned_memory_cache, 1);
        assert_eq!(cache.stats().hits_memory_cache, 1);
        assert_eq!(cache.stats().hits_fs_cache, 2);
        assert_eq!(cache.stats().misses, 0);

        // init
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
        let response = call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();
        assert_eq!(response.messages.len(), 0);

        // execute
        let info = mock_info("verifies", &coins(15, "earth"));
        let msg = br#"{"release":{}}"#;
        let response = call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();
        assert_eq!(response.messages.len(), 1);
    }
}

pub fn main() {
    loop {
        run_threads(vec![
            call_instantiate_on_cached_contract,
            call_execute_on_cached_contract,
        ]);
    }
}

fn run_threads(functions: Vec<fn() -> ()>) {
    const TARGET_THREAD_COUNT: usize = 20;
    let multiplier = if functions.len() < TARGET_THREAD_COUNT {
        TARGET_THREAD_COUNT / functions.len()
    } else {
        1
    };
    let mut threads = Vec::with_capacity(functions.len() * multiplier);

    for _ in 0..multiplier {
        for function in &functions {
            let thread = std::thread::spawn(*function);
            threads.push(thread);
        }
    }

    for thread in threads {
        thread.join().unwrap();
    }
}
