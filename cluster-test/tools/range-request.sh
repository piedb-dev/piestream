#!/bin/bash

#-X GET \

#-H "authorization: AWS4-HMAC-SHA256 Credential=hummockadmin/20220819/custom/s3/aws4_request, SignedHeaders=host;range;x-amz-content-sha256;x-amz-date;x-amz-user-agent, Signature=61c598a40c099e0a403ff2a0620012ad8cc73da6d5ff5e56303bb926a3b8f8b2" \

curl -v -o /tmp/curl.response.data \
http://122.70.153.21:9000/hummock001/hummock_001/128086.data?x-id=GetObject \
-H "range: bytes=0-16392" \
-H "user-agent: aws-sdk-rust/0.12.0 os/linux lang/rust/1.63.0-nightly" \
-H "x-amz-user-agent: aws-sdk-rust/0.12.0 api/s3/0.12.0 os/linux lang/rust/1.63.0-nightly" \
-H "x-amz-date: 20220819T171743Z" \
-H "x-amz-content-sha256: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855" \
-H "host: 122.70.153.21:9000" 
