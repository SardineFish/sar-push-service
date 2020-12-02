# Sar Push Service API Documentation

## API Content
- [Sar Push Service API Documentation](./index.md)
  - [User and Access Control Service](./access.md)
  - [Services Profile Management](./services.md)
  - [Email Notify Service](./notify.md)


## Authorization

All requests to the API require authorization with HTTP basic authentication. With the `uid` as the username and `secret` as password.

### Example in JS

```js
const uid = "<uid>";
const secret = "<secret>";
fetch("http://<host>/path/to/api", {
    headers: {
        "Authorization": `Basic ${btoa(uid + ":" + secret)}`
    },
});
```


## Data Scheme

All data is sent & received in JSON.

JSON data should be sent with method `POST`, `PUT`, `UPDATE` and `PATCH` and header `Content-Type` set to `application/json`.

Response with status code `4XX` and `5XX` will always has a body in JSON discribing the error infomations in scheme bellow.

```json
{
    "error": "<Error message>"
}
``` 


## Service Profile

**Sar Push Service** is combined with numbers of sub `Service`, each handles a small part of the whole push service.

For example, the `Access` service manipulate the users and their `secret` key, the `EmailNotify` service handles the email notify request.

Each service has a profile for a individual user describing its settings and permissions. An API request by authorized user works appropriately only if the user has a valid profile of the corresponding service.
