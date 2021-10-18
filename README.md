# Getting Started

## Get Object async
```rust
let oss_instance = OSS::new("your_AccessKeyId", "your_AccessKeySecret", "your_Endpoint", "your_Bucket");
let buf = oss_instance.get_object("objectName", None, None).await;
String::from_utf8(buf)?
```

## Put Object by file
```rust
let filename = "filename";
let oss_instance = OSS::new("your_AccessKeyId", "your_AccessKeySecret", "your_Endpoint", "your_Bucket");
let result = oss_instance.put_object_from_file(filename, "object", None, None).await;
assert_eq!(result.is_ok(), true)
```

## MultipartUpload for big file 
```rust
let filename = "filename";
let oss_instance = OSS::new("your_AccessKeyId", "your_AccessKeySecret", "your_Endpoint", "your_Bucket");
let object_name = "object_name";
let file = "/tmp/tmp.txt";
let chunk_size = 102400;

let res = oss_instance.chunk_upload_by_size(object_name, file, chunk_size).await;
println!("res: {:?}", res);
```

## Delete Ojbect
```rust
let oss_instance = OSS::new("your_AccessKeyId", "your_AccessKeySecret", "your_Endpoint", "your_Bucket");
let result = oss_instance.delete_object("object").await;
assert_eq!(result.is_ok(), true)
```

