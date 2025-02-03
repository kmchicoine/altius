To run:
'cargo run' in altius directory

To test:
'curl -X PUT -d 'new_value' http://127.0.0.1:7878/test'
'curl http://127.0.0.1:7878/test'

For simplicity sake, the database is implemented as an in-memory hashmap of key:value pairs. The endpoint acts as the key, and the body of the request is the value. In the above example, the key:value pair is test:new_value. 

If the same endpoint receives a second PUT request, the existing value is overwritten with the new value.

If a GET request is made before a PUT request, the server returns a "key not found" error.

Known limitations:
-Request size is limited to 1024 bytes.
-Only GET and PUT are supported.
-(more of an FYI than a limitation) I modified the simple server code from the end of the Rust book to use HTTPARSE, handle requests, and have the mock database.