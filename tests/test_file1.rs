
extern crate bitcrust_lib;

extern crate byteorder;


use std::io::BufReader;
use std::fs::File;

mod blk_file;

use std::time::{Instant};
extern crate rayon;


#[test]
#[ignore]
fn load_file1() {

    let mut store = bitcrust_lib::init();

    let fileno = 0;
    let name = format!("./data/blk{:05}.dat", fileno);
    println!("Processing {}", name);
    let f = File::open(name).unwrap();
    let mut rdr = BufReader::new(f);

    let mut blocks = 0;
    loop {
        let blk = blk_file::read_block(&mut rdr).unwrap();

        if blk.is_none() {
            break;
        }

        bitcrust_lib::add_block(&mut store, &blk.unwrap());

        blocks += 1;




        if blocks == 2 {
            break;
        }

    }




}





#[test]
#[ignore]
fn load_file_large() {
    let start = Instant::now();
    const BLOCK_COUNT: u64 = 450000;

    let mut blocks = 0;

    let mut store = bitcrust_lib::init();

    store.initial_sync = true;
    for fileno in 0..999 {
        let name = format!("./core-blocks/blk{:05}.dat", fileno);
        println!("Processing {}", name);
        let f = File::open(name).unwrap();
        let mut rdr = BufReader::new(f);

        loop {
            let blk = blk_file::read_block(&mut rdr).unwrap();

            if blk.is_none() {
                break;
            }

            bitcrust_lib::add_block(&mut store, &blk.unwrap());


            blocks += 1;

            if blocks % 100 == 0 {
                println!("Processed {} blocks in {} sec at {}/s", blocks, start.elapsed().as_secs(),
                         blocks / (start.elapsed().as_secs() + 1));
            }

            if blocks >= BLOCK_COUNT {
                break;
            }
        }

        if blocks >= BLOCK_COUNT {
            break;
        }
    }

    println!("DONE: Processed {} blocks in {} sec at {}/s", blocks, start.elapsed().as_secs(),
             blocks / (start.elapsed().as_secs() + 1));
}

use std::thread;
#[test]
#[ignore]
fn load_large_concurrent() {
    const THREADS: usize = 5;
    const BLOCK_COUNT: u64 = 400000;



    let handles: Vec<_> = (0..THREADS).map(|n| {
        //let mut store = store_b.clone();
        thread::spawn(move || {
            let mut store = bitcrust_lib::init_prs();

            let start = Instant::now();
            let mut blocks = 0;
            for fileno_b in 0..999 {
                let fileno = fileno_b * THREADS + n;
                let name = format!("./data/blk{:05}.dat", fileno);
                println!("Processing {}", name);
                let f = File::open(name).unwrap();
                let mut rdr = BufReader::new(f);

                loop {
                    let blk = blk_file::read_block(&mut rdr).unwrap();

                    if blk.is_none() {
                        break;
                    }

                    bitcrust_lib::add_block(&mut store, &blk.unwrap());


                    blocks += 1;

                    if blocks % 100 == 0 {
                        println!("Processed {} blocks in {} sec at {}/s", blocks, start.elapsed().as_secs(),
                                 blocks / (start.elapsed().as_secs() + 1));
                    }

                    if blocks >= BLOCK_COUNT {
                        break;
                    }
                }

                if blocks >= BLOCK_COUNT {
                    break;
                }
            }
        })
    }).collect();

    for h in handles {
        h.join().unwrap();
    }
}