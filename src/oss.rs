use super::errors::Error;
use bytes::Bytes;
use chrono::prelude::*;
use quick_xml::{events::Event, Reader};
use reqwest::header::{HeaderMap, CONTENT_LENGTH, DATE};
use reqwest::Client;
use std::collections::HashMap;
use std::str;

use crate::bucket::{Bucket, ListBuckets};
use crate::errors::ObjectError;

use super::auth::*;
use super::utils::*;

#[derive(Clone, Debug)]
pub struct OSS {
    key_id: String,
    key_secret: String,
    endpoint: String,
    bucket: String,
    pub client: Client,
}

const RESOURCES: [&str; 50] = [
    "acl",
    "uploads",
    "location",
    "cors",
    "logging",
    "website",
    "referer",
    "lifecycle",
    "delete",
    "append",
    "tagging",
    "objectMeta",
    "uploadId",
    "partNumber",
    "security-token",
    "position",
    "img",
    "style",
    "styleName",
    "replication",
    "replicationProgress",
    "replicationLocation",
    "cname",
    "bucketInfo",
    "comp",
    "qos",
    "live",
    "status",
    "vod",
    "startTime",
    "endTime",
    "symlink",
    "x-oss-process",
    "response-content-type",
    "response-content-language",
    "response-expires",
    "response-cache-control",
    "response-content-disposition",
    "response-content-encoding",
    "udf",
    "udfName",
    "udfImage",
    "udfId",
    "udfImageDesc",
    "udfApplication",
    "comp",
    "udfApplicationLog",
    "restore",
    "callback",
    "callback-var",
];

impl OSS {
    pub fn new(key_id: String, key_secret: String, endpoint: String, bucket: String) -> Self {
        OSS {
            key_id: key_id,
            key_secret: key_secret,
            endpoint: endpoint,
            bucket: bucket,
            client: reqwest::Client::new(),
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    pub fn key_secret(&self) -> &str {
        &self.key_secret
    }

    pub fn set_bucket(&mut self, bucket: &str) {
        self.bucket = bucket.to_string()
    }

    pub fn host(&self, bucket: &str, object: &str, resources_str: &str) -> String {
        if self.endpoint.starts_with("https") {
            format!(
                "https://{}.{}/{}?{}",
                bucket,
                self.endpoint.replacen("https://", "", 1),
                object,
                resources_str
            )
        } else {
            format!(
                "http://{}.{}/{}?{}",
                bucket,
                self.endpoint.replacen("http://", "", 1),
                object,
                resources_str
            )
        }
    }

    pub fn date(&self) -> String {
        let now: DateTime<Utc> = Utc::now();
        now.format("%a, %d %b %Y %T GMT").to_string()
    }

    pub fn get_resources_str<S>(&self, params: HashMap<S, Option<S>>) -> String
    where
        S: AsRef<str>,
    {
        let mut resources: Vec<(&S, &Option<S>)> = params
            .iter()
            .filter(|(k, _)| RESOURCES.contains(&k.as_ref()))
            .collect();
        resources.sort_by(|a, b| a.0.as_ref().to_string().cmp(&b.0.as_ref().to_string()));
        let mut result = String::new();
        for (k, v) in resources {
            if !result.is_empty() {
                result += "&";
            }
            if let Some(vv) = v {
                result += &format!("{}={}", k.as_ref().to_owned(), vv.as_ref());
            } else {
                result += k.as_ref();
            }
        }
        result
    }

    pub async fn list_bucket<S, R>(&self, resources: R) -> Result<ListBuckets, Error>
    where
        S: AsRef<str>,
        R: Into<Option<HashMap<S, Option<S>>>>,
    {
        let resources_str = if let Some(r) = resources.into() {
            self.get_resources_str(r)
        } else {
            String::new()
        };
        let host = self.endpoint();
        let date = self.date();

        let mut headers = HeaderMap::new();
        headers.insert(DATE, date.parse()?);
        let authorization = self.oss_sign(
            "GET",
            self.key_id(),
            self.key_secret(),
            "",
            "",
            &resources_str,
            &headers,
        );
        headers.insert("Authorization", authorization.parse()?);

        let resp = self.client.get(host).headers(headers).send().await?;

        let xml_str = resp.text().await?;
        let mut result = Vec::new();
        let mut reader = Reader::from_str(xml_str.as_str());
        reader.trim_text(true);
        let mut buf = Vec::new();

        let mut prefix = String::new();
        let mut marker = String::new();
        let mut max_keys = String::new();
        let mut is_truncated = false;
        let mut next_marker = String::new();
        let mut id = String::new();
        let mut display_name = String::new();

        let mut name = String::new();
        let mut location = String::new();
        let mut create_date = String::new();
        let mut extranet_endpoint = String::new();
        let mut intranet_endpoint = String::new();
        let mut storage_class = String::new();

        let list_buckets;

        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Start(ref e)) => match e.name() {
                    b"Prefix" => prefix = reader.read_text(e.name(), &mut Vec::new())?,
                    b"Marker" => marker = reader.read_text(e.name(), &mut Vec::new())?,
                    b"MaxKeys" => max_keys = reader.read_text(e.name(), &mut Vec::new())?,
                    b"IsTruncated" => {
                        is_truncated = reader.read_text(e.name(), &mut Vec::new())? == "true"
                    }
                    b"NextMarker" => next_marker = reader.read_text(e.name(), &mut Vec::new())?,
                    b"ID" => id = reader.read_text(e.name(), &mut Vec::new())?,
                    b"DisplayName" => display_name = reader.read_text(e.name(), &mut Vec::new())?,

                    b"Bucket" => {
                        name = String::new();
                        location = String::new();
                        create_date = String::new();
                        extranet_endpoint = String::new();
                        intranet_endpoint = String::new();
                        storage_class = String::new();
                    }

                    b"Name" => name = reader.read_text(e.name(), &mut Vec::new())?,
                    b"CreationDate" => create_date = reader.read_text(e.name(), &mut Vec::new())?,
                    b"ExtranetEndpoint" => {
                        extranet_endpoint = reader.read_text(e.name(), &mut Vec::new())?
                    }
                    b"IntranetEndpoint" => {
                        intranet_endpoint = reader.read_text(e.name(), &mut Vec::new())?
                    }
                    b"Location" => location = reader.read_text(e.name(), &mut Vec::new())?,
                    b"StorageClass" => {
                        storage_class = reader.read_text(e.name(), &mut Vec::new())?
                    }
                    _ => (),
                },
                Ok(Event::End(ref e)) if e.name() == b"Bucket" => {
                    let bucket = Bucket::new(
                        name.clone(),
                        create_date.clone(),
                        location.clone(),
                        extranet_endpoint.clone(),
                        intranet_endpoint.clone(),
                        storage_class.clone(),
                    );
                    result.push(bucket);
                }
                Ok(Event::Eof) => {
                    list_buckets = ListBuckets::new(
                        prefix,
                        marker,
                        max_keys,
                        is_truncated,
                        next_marker,
                        id,
                        display_name,
                        result,
                    );
                    break;
                } // exits the loop when reaching end of file
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                _ => (), // There are several other `Event`s we do not consider here
            }
            buf.clear();
        }
        Ok(list_buckets)
    }

    pub async fn get_object<S>(
        &self,
        object: S,
        headers: Option<HashMap<S, S>>,
        resources: Option<HashMap<S, Option<S>>>,
    ) -> Result<Bytes, reqwest::Error>
    where
        S: AsRef<str>,
    {
        let object = object.as_ref();
        let resources_str = if let Some(r) = resources {
            self.get_resources_str(r)
        } else {
            String::new()
        };
        let host = self.host(self.bucket(), object, &resources_str);
        let date = self.date();
        let mut headers = if let Some(h) = headers {
            to_headers(h).unwrap()
        } else {
            HeaderMap::new()
        };
        headers.insert(DATE, date.parse().unwrap());
        let authorization = self.oss_sign(
            "GET",
            self.key_id(),
            self.key_secret(),
            self.bucket(),
            object,
            &resources_str,
            &headers,
        );
        headers.insert("Authorization", authorization.parse().unwrap());

        let res = reqwest::Client::new()
            .get(&host)
            .headers(headers)
            .send()
            .await?;
        Ok(res.bytes().await?)
    }

    pub async fn head_object<S>(
        &self,
        object: S,
        headers: Option<HashMap<S, S>>,
        resources: Option<HashMap<S, Option<S>>>,
    ) -> Result<HeaderMap, reqwest::Error>
    where
        S: AsRef<str>,
    {
        let object = object.as_ref();
        let resources_str = if let Some(r) = resources {
            self.get_resources_str(r)
        } else {
            String::new()
        };
        let host = self.host(self.bucket(), object, &resources_str);
        let date = self.date();
        let mut headers = if let Some(h) = headers {
            to_headers(h).unwrap()
        } else {
            HeaderMap::new()
        };
        headers.insert(DATE, date.parse().unwrap());
        let authorization = self.oss_sign(
            "HEAD",
            self.key_id(),
            self.key_secret(),
            self.bucket(),
            object,
            &resources_str,
            &headers,
        );
        headers.insert("Authorization", authorization.parse().unwrap());

        let res = reqwest::Client::new()
            .head(&host)
            .headers(headers)
            .send()
            .await?;
        Ok(res.headers().clone())
    }

    pub async fn put_object_from_buffer<S1, S2, H, R>(
        &self,
        buf: &[u8],
        object: S1,
        headers: H,
        resources: R,
    ) -> Result<Bytes, reqwest::Error>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        H: Into<Option<HashMap<S2, S2>>>,
        R: Into<Option<HashMap<S2, Option<S2>>>>,
    {
        let object = object.as_ref();
        let resources_str = if let Some(r) = resources.into() {
            self.get_resources_str(r)
        } else {
            String::new()
        };
        let host = self.host(self.bucket(), object, &resources_str);
        let date = self.date();

        let mut headers = if let Some(h) = headers.into() {
            to_headers(h).unwrap()
        } else {
            HeaderMap::new()
        };
        headers.insert(DATE, date.parse().unwrap());
        let authorization = self.oss_sign(
            "PUT",
            self.key_id(),
            self.key_secret(),
            self.bucket(),
            object,
            &resources_str,
            &headers,
        );
        headers.insert("Authorization", authorization.parse().unwrap());

        let res = reqwest::Client::new()
            .put(&host)
            .headers(headers)
            .body(buf.to_owned())
            .send()
            .await?;
        Ok(res.bytes().await?)
    }

    pub async fn put_object_from_file<S1, S2, S3, H, R>(
        &self,
        file: S1,
        object_name: S2,
        headers: H,
        resources: R,
    ) -> Result<(), Error>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        S3: AsRef<str>,
        H: Into<Option<HashMap<S3, S3>>>,
        R: Into<Option<HashMap<S3, Option<S3>>>>,
    {
        let object_name = object_name.as_ref();
        let resources_str = if let Some(r) = resources.into() {
            self.get_resources_str(r)
        } else {
            String::new()
        };
        let host = self.host(self.bucket(), object_name, &resources_str);
        let date = self.date();
        let buf = load_file(file)?;
        let mut headers = if let Some(h) = headers.into() {
            to_headers(h)?
        } else {
            HeaderMap::new()
        };
        headers.insert(DATE, date.parse()?);
        headers.insert(CONTENT_LENGTH, buf.len().to_string().parse()?);
        let authorization = self.oss_sign(
            "PUT",
            self.key_id(),
            self.key_secret(),
            self.bucket(),
            object_name,
            &resources_str,
            &headers,
        );
        headers.insert("Authorization", authorization.parse()?);

        let resp = self
            .client
            .put(&host)
            .headers(headers)
            .body(buf)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(Error::Object(ObjectError::PutError {
                msg: format!("can not put object, status code: {}", resp.status()).into(),
            }))
        }
    }

    pub async fn delete_object<S>(&self, object_name: S) -> Result<(), Error>
    where
        S: AsRef<str>,
    {
        let object_name = object_name.as_ref();
        let host = self.host(self.bucket(), object_name, "");
        let date = self.date();

        let mut headers = HeaderMap::new();
        headers.insert(DATE, date.parse()?);
        let authorization = self.oss_sign(
            "DELETE",
            self.key_id(),
            self.key_secret(),
            self.bucket(),
            object_name,
            "",
            &headers,
        );
        headers.insert("Authorization", authorization.parse()?);

        let resp = self.client.delete(&host).headers(headers).send().await?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(Error::Object(ObjectError::DeleteError {
                msg: format!("can not delete object, status code: {}", resp.status()).into(),
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_oss() {
        let oss_instance = OSS::new(
            "key".to_string(),
            "secret".to_string(),
            "xxxx.aliyuncs.com".to_string(),
            "xxxx".to_string(),
        );

        put_object(&oss_instance).await;
        get_object(&oss_instance).await;
        delete_object(&oss_instance).await;
    }

    async fn put_object(oss_instance: &OSS) {
        let result = oss_instance
            .put_object_from_file(
                "/xxxxx/Cargo.toml",
                "objectName",
                None::<HashMap<&str, &str>>,
                None,
            )
            .await;
        assert_eq!(result.is_ok(), true);
    }

    async fn get_object(oss_instance: &OSS) {
        let result = oss_instance
            .get_object("objectName", None::<HashMap<&str, &str>>, None)
            .await;
        assert_eq!(result.is_ok(), true);
        println!("text = {:?}", String::from_utf8(result.unwrap().to_vec()));
    }
    async fn delete_object(oss_instance: &OSS) {
        let result = oss_instance.delete_object("objectName").await;
        assert_eq!(result.is_ok(), true);
    }
}
