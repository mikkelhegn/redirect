# Spin Redirect app

A nice litte app to create shortlinks, and generate QR codes.

There's an admin UI at `/admin/index.html`

The redirect component reades the destination address from the KV store, and sends a 307 temporary redirect with the Location header set to destination.

`http://localhost:3000?UrzqN3Gi`

The api `/api` is a CRUD api to manage entries in the KV store:

```
Get a record

curl .../api?UrzqN3Gi

Get all records

curl .../api

Create a record

curl -X POST --data ''{"name": "fermyon.com", "url": "https://www.fermyon.com"}'' .../api

Delete a record

curl -X DELETE ../api?UrzqN3Gi
```
