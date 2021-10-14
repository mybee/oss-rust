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

// init multi upload
let upload_id = oss_instance.initiate_multipart_upload(object_name, None::<HashMap<&str, &str>>).await.unwrap();
// chunk object by size
let chunks = split_file_by_part_size(file, 1024).await.unwrap();
// part upload chunks
let mut parts = vec![];
for chunk in chunks {
    let etag = oss_instance.upload_part(file,object_name,chunk.clone(),upload_id.clone(),None::<HashMap<&str, &str>>).await.unwrap();
    parts.push(Part {PartNumber: chunk.number,ETag: etag,});
}
// complete multi upload
let res = oss_instance.complete_multipart_upload(object_name,upload_id,CompleteMultipartUpload { Part: parts },None::<HashMap<&str, &str>>).await;
```

## Delete Ojbect
```rust
let oss_instance = OSS::new("your_AccessKeyId", "your_AccessKeySecret", "your_Endpoint", "your_Bucket");
let result = oss_instance.delete_object("object").await;
assert_eq!(result.is_ok(), true)
```

