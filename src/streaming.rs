use std::{sync::{Arc, Weak}, time::Duration};

use rusb_async::TransferPool;

use crate::easycap::*;

impl EasyCap {
    pub fn test(&self) {
        println!("Hello, World");
    }

    pub fn begin_streaming(&self) {
        
        let mut async_pool = TransferPool::new(self.handle.clone()).expect("Failed to create async pool!");
        
        const NUM_TRANSFERS: usize = 20;
        const BUF_SIZE: usize = 0x6000;

        while async_pool.pending() < NUM_TRANSFERS {
            async_pool
                .submit_iso(0x81, Vec::with_capacity(BUF_SIZE), 8)
                .expect("Failed to submit transfer");
        }

        let timeout = Duration::from_secs(10);
        loop {
            let data = async_pool.poll(timeout).expect("Transfer failed");
            // just getting 0 for some reason? not handling isos right in lib?
            println!("Got data: {} {:?}", data.len(), data);
            async_pool
                .submit_iso(0x81, data, 8)
                .expect("Failed to resubmit transfer");
        }
        
    }
}