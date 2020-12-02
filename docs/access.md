# User and Access Control

All request without permission will get a `403` response with error.

## Add a user

`POST /access/user`

*Require access of `UserAdmin`

### Request

```json
{
    "name": "<Name of the user>",
    "description": "<Full description of a user>"
}
```

### Response

The `uid` returned is the identify of a user. The `secret` is used to authorize the user, `secret` MUST be keep secret and nolonger obtainable from any API. the `secret` can only be revoke and regenerate for a new one.

```json
{
    "uid": "<Random string>",
    "secret": "<Random string>"
}
```

### Errors

Since duplicate `name` and `description` with other user is allowed, this request should always success with different `uid` and `secret` generated.

----------------

## Get a user profile
`GET /access/user/{uid}`

Can only get profile of the user himself normally. 
Only users with access of `UserAdmin` are able to get others' profile.

### Request
No request data needed.

### Response

```json
{
    "name": "<Name of the user>",
    "description": "<Full description of the user>",
    "uid": "<uid>"
}
```

### Errors
If the user not exists, an error with status code `404` will be responsed.

----------------

## Update user profile

`PATCH /access/user/{uid}`

Can only update profile of the user himself normally. 
Only users with access of `UserAdmin` are able to update others' profile.

### Request
User profile can partially update, fill the request with fields to change and leave others null.

Example:
```json
{
    "name": null,
    "description": "Discription string to change",
}
```

### Errors
If the user not exists, an error with status code `404` will be responsed.

----------------

## Revoke and regenerate user secret
`POST /access/user/{uid}/secret`

Normal user can only revoke their own `secret`. Only users with access of `UserAdmin` are able to revoke others' `secret`

### Request
No request data required.

### Response
```json
{
    "uid": "<uid>",
    "secret": "<A new secret generated>"
}
```
### Errors
If the user not exists, an error with status code `404` will be responsed.

----------------

## Delete a user
`DELETE /access/user/{uid}`

Only available for users with access of `UserAdmin`.

### Request
No request data required.

### Response
If the user not exists, an empty response with status code `204` will return.
If the user exists and successfully deleted, a response in `200` with data in folowing scheme will return.
```json
{
    "name": "<Name of the user>",
    "description": "<Full description of the user>",
    "uid": "<uid>"
}
```

----------------