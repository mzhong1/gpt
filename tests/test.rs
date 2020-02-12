use simplelog;

use uuid;

use gpt::disk;
use gpt::header::{read_header, write_header, Header};
use gpt::partition::{read_partitions, Partition};
use gpt::partition_types::Type;
use simplelog::{Config, SimpleLogger};
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use tempfile::NamedTempFile;

#[test]
fn test_read_header() {
    let expected_header = Header {
        signature: "EFI PART".to_string(),
        revision: 65536,
        header_size_le: 92,
        crc32: 1050019802,
        reserved: 0,
        current_lba: 1,
        backup_lba: 95,
        first_usable: 34,
        last_usable: 62,
        disk_guid: uuid::Uuid::from_str("f12fc858-c753-41d3-93a4-bfac001cdf9f").unwrap(),
        part_start: 2,
        num_parts: 128,
        part_size: 128,
        crc32_parts: 151952294,
    };

    let expected_partition = Partition {
        part_type_guid: gpt::partition_types::LINUX_FS,
        part_guid: uuid::Uuid::from_str("6fcc8240-3985-4840-901f-a05e7fd9b69d").unwrap(),
        first_lba: 34,
        last_lba: 62,
        flags: 0,
        name: "primary".to_string(),
    };

    let diskpath = Path::new("tests/fixtures/gpt-linux-disk-01.img");
    let h = read_header(diskpath, disk::DEFAULT_SECTOR_SIZE).unwrap();

    println!("header: {:?}", h);
    assert_eq!(h, expected_header);

    let p = read_partitions(diskpath, &h, disk::DEFAULT_SECTOR_SIZE).unwrap();
    println!("Partitions: {:?}", p);
    assert_eq!(*p.get(&1).unwrap(), expected_partition);
}

#[test]
fn test_write_header() {
    let _ = SimpleLogger::init(simplelog::LevelFilter::Trace, Config::default());
    let mut tempdisk = NamedTempFile::new().expect("failed to create tempfile disk");
    {
        let data: [u8; 4096] = [0; 4096];
        println!("Creating blank header file for testing");
        for _ in 0..100 {
            tempdisk.as_file_mut().write_all(&data).unwrap();
        }
    };
    println!("Writing header");
    let w = write_header(
        tempdisk.path(),
        Some(uuid::Uuid::from_str("f400b934-48ef-4381-8f26-459f6b33c7df").unwrap()),
        disk::DEFAULT_SECTOR_SIZE,
    );
    println!("Wrote header: {:?}", w);
    println!("Reading header");
    let h = read_header(tempdisk.path(), disk::DEFAULT_SECTOR_SIZE).unwrap();
    println!("header: {:#?}", h);

    let p = Partition {
        part_type_guid: Type::from_str("0FC63DAF-8483-4772-8E79-3D69D8477DE4").unwrap(),
        part_guid: uuid::Uuid::new_v4(),
        first_lba: 36,
        last_lba: 40,
        flags: 0,
        name: "gpt test".to_string(),
    };
    p.write(tempdisk.path(), 0, h.part_start, disk::DEFAULT_SECTOR_SIZE)
        .unwrap();
}

#[test]
fn test_partition_type_fromstr() {
    assert_eq!(
        gpt::partition_types::Type::from_str("933AC7E1-2EB4-4F13-B844-0E14E2AEF915").unwrap(),
        gpt::partition_types::LINUX_HOME
    );
    assert_eq!(
        gpt::partition_types::Type::from_str("114EAFFE-1552-4022-B26E-9B053604CF84").unwrap(),
        gpt::partition_types::ANDROID_BOOTLOADER2
    );
    assert_eq!(
        gpt::partition_types::Type::from_str("00000000-0000-0000-0000-000000000000").unwrap(),
        gpt::partition_types::UNUSED
    );
}

#[test]
fn test_read_fdisk() {
    let expected_header = Header {
        signature: "EFI PART".to_string(),
        revision: 65536,
        header_size_le: 92,
        crc32: 2605617263,
        reserved: 0,
        current_lba: 1,
        backup_lba: 204799,
        first_usable: 2048,
        last_usable: 204766,
        disk_guid: uuid::Uuid::from_str("4ec73dcd-4b5a-5045-9d74-d36c12f6d9a3").unwrap(),
        part_start: 2,
        num_parts: 128,
        part_size: 128,
        crc32_parts: 2214351575,
    };
    let diskpath = Path::new("tests/fixtures/fdisk.img");
    let h = read_header(diskpath, disk::DEFAULT_SECTOR_SIZE).unwrap();

    println!("header: {:?}", h);
    assert_eq!(h, expected_header);

    let expected_partitions = [
        Partition {
            part_type_guid: gpt::partition_types::LINUX_FS,
            part_guid: uuid::Uuid::from_str("63ae360b-16b7-4e43-9a20-263d50df5554").unwrap(),
            first_lba: 2048,
            last_lba: 2067,
            flags: 0,
            name: "".to_string(),
        },
        Partition {
            part_type_guid: gpt::partition_types::LINUX_FS,
            part_guid: uuid::Uuid::from_str("2281aca4-cbb4-f140-9331-a5a406c97a51").unwrap(),
            first_lba: 4096,
            last_lba: 4195,
            flags: 0,
            name: "".to_string(),
        },
    ];
    let p = read_partitions(diskpath, &h, disk::DEFAULT_SECTOR_SIZE).unwrap();
    println!("Partitions: {:?}", p);
    assert_eq!(*p.get(&1).unwrap(), expected_partitions[0]);
    assert_eq!(*p.get(&2).unwrap(), expected_partitions[1]);
}
