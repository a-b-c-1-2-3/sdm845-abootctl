use std::path::Path;
use gpt::partition::{Partition};
use gpt::disk;
use std::collections::BTreeMap;
use std::io;

fn get_partitions_for_dev(dev_path: &Path) -> Result<BTreeMap<u32, Partition>, io::Error> {

	println!("Testing path {}", dev_path.display());
	let size = disk::LogicalBlockSize::Lb4096;
	let header = gpt::header::read_header(dev_path, size);
	match header {
		Ok(header) => gpt::partition::read_partitions(dev_path, &header, size),
		Err(e) => Err(e),
	}
}

pub fn get_boot_partitions() -> (Partition, Partition, String) {
	//Open relevant GPT stuff

	let blockdevs = block_utils::get_block_devices();
	assert!(!blockdevs.is_err());

	let blockdevs = blockdevs.unwrap();

	let dev_path = blockdevs.into_iter()
		.find(|dev_path| { match get_partitions_for_dev(dev_path.as_path()) {
			Ok(dev) => dev.values().find(|p| p.name == "boot_a").is_some(),
			Err(e) => false,
		}});
	if dev_path.is_none() {
		panic!("Failed to find boot_a, does this device have slots?");
	}
	let dev_path = dev_path.unwrap();

	let all_partitions =  get_partitions_for_dev(dev_path.as_path()).unwrap();
	let boot_a = all_partitions.values().find(|x| x.name == "boot_a").unwrap().clone();
	let boot_b = all_partitions.values().find(|x| x.name == "boot_b").unwrap().clone();

	// for dev_path in blockdevs {
	// 	let header = gpt::header::read_header(dev_path.as_path(), size).unwrap();
	// 	let mut partitions = gpt::partition::read_partitions(dev_path.as_path(), &header, size).unwrap();
	// 	let res = partitions.values().try_find(|x| x.name == "boot_a");
	// 	if res.is_err() {
	// 		continue;
	// 	}
	// }

	// let boot_a = partitions.values().find(|x| x.name == "boot_a").unwrap();

	return (boot_a, boot_b, dev_path.as_path().to_str().unwrap().to_string());
}

pub fn set_boot_partitions(boot_a: Partition, boot_b: Partition, path: String) {

    //Opens relevant stuff
    let path = Path::new(&path);
    let config = gpt::GptConfig::new();
    let config = config.writable(true); //config needs to be shadowed here for some reason
    let mut disk = config.open(&path).unwrap(); //Should be fine since for this function to run get_boot_partitions() must have succeeded
    let part_table = disk.partitions();
    let mut new_part_table = part_table.clone();

    for (key, part) in part_table.iter() {
        if part.name == "boot_a" {
            new_part_table.insert(*key, boot_a.clone());
        }
        else if part.name == "boot_b" {
            new_part_table.insert(*key, boot_b.clone());
        }
    }

    disk.update_partitions(new_part_table);

    let _final_res = disk.write_inplace();
    match _final_res {
        Ok(_final_res) => println!("Successfully wrote changes to disk"),
        Err(e) => panic!("{}", e),
    }
}