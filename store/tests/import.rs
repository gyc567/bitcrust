

extern crate store;


use std::time::Instant;

mod util;
mod blk_file;



#[test]
#[ignore]
fn test_import() {

    let mut db = store::init_empty("tst-import").unwrap();
    let mut orphans = std::collections::HashMap::new();
    let now = Instant::now();
    let mut blocks = 0;

    for blk in blk_file::read_blocks() {

        // we only use the header
        let mut raw_hdr = blk;
        let mut hash = store::double_sha256(&raw_hdr[0..80]);

        let header = store::Header::new(&raw_hdr[0..80]).unwrap();
        let prev_hash = header.prev_hash;


        let add_result = store::header_add(&mut db, &hash, header).unwrap();
        if let store::HeaderAddResult::Orphan(parent) = add_result {

            let header = store::Header::new(&raw_hdr[0..80]).unwrap();


            //println!("Marking orphan {} waiting for {}", util::to_hex_rev(&hash[..]), util::to_hex_rev(&parent[..]));
            assert!(orphans.insert(parent, (hash,header)).is_none());


        } else {
            //println!("Added header {} with prev {}", util::to_hex_rev(&hash[..]), util::to_hex_rev(&prev_hash[..]));


            while let Some(&(ref orphan_hash, ref orphan_header)) = orphans.get(&hash) {
                //println!("Adding decendent {} of {}", util::to_hex_rev(&orphan_hash[..]), util::to_hex_rev(&hash[..]));
                let add_result = store::header_add(&mut db, &orphan_hash, orphan_header.clone()).unwrap();

                if let store::HeaderAddResult::Orphan(_) = add_result {
                    panic!("{} should not be orphan anymore", util::to_hex_rev(&hash[..]));
                }
                hash = *orphan_hash;
            }
        }

        blocks += 1;
    }

    let elapsed = now.elapsed().as_secs() * 1000 + now.elapsed().subsec_nanos() as u64 / 1000_000;
    let ms_block = elapsed / blocks;
    println!("Processed {} in {} ms,  {} ms/block", blocks, elapsed, ms_block);

    let _ = store::header_get(&mut db,
                                &util::hash_from_hex("000000000000034a7dedef4a161fa058a2d67a173a90155f3a2fe6fc132e0ebf"))
        .unwrap().unwrap();


}

