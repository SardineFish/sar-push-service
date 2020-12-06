# Services Profile Management

A service profile describing the settings and permissions of a service module in **Sar Push Service**.

The *Service Profile Management* service manage the service profiles of every user.

All request in this service without a permission will get a `403` response with error.

## Service Profile
**This section describing the service profile data scheme in JSON format, used for storage the settings and permissions of a service. Every API doc of a service will have this section.*

**Here is the service profile data scheme for the **Service Profile Management** service.*
```json
{
    "access": "<User | Admin | Root>"
}
```

----------------

## List services profile of a given user
`GET /service/profile/{uid}`

Normally, a user can list the services profile of himself. Only the user with access of `Admin` can list profiles of other users.

### Request
No data required.

### Response
The service profile data can be different between different service. The profile data scheme will be described in API docs of each service.
```json
{
    "services": [
        {
            "service_id": "<A service profile ID>",
            "service": {
                "type": "<Type name of the service>",
                "profile": { /* ... */ }
            }
        },
        {
            "service_id": "<Another service profile ID>",
            "service": {
                "type": "<Another Type name of the service>",
                "profile": { /* ... */ }
            }
        },
        // Other service profiles...
    ]
}
```

### Errors
If the user not exists, an error with status code `404` will be responsed.

----------------

## Add service profile
`POST /service/profile/{uid}`

Only user with access of `Admin` are permit to add service profiles to any other user.

### Request
The profile data scheme should match the corresponding service.
```json
{
    "type": "<Type name of the service>",
    "profile": { /* ... */ }
}
```

### Resposne
```json
{
    "service_id": "<The service profile ID>",
    "service": {
        "type": "<The Type name of the service>",
        "profile": { /* ... */ }
    }
},
```

### Errors
- If the user not exists, an error with status code `404` will be responsed.
- If the data not match the scheme of specific service profile, an error `400` will be responsed.
- If a service with the same type already exists, an error with `409` will be responsed.

----------------

## Update service profile
`PATCH /service/profile/{user}/{service_id}`

### Request
The service profile can be partially updated. Fill the profile with fields to change and leave other fields with `null`. The data scheme should match the corresponding service.
```json
{
    "type": "Type of the Service",
    "profile": {
        "_": "// The profile content of the service to change..."
    }
    
}
```

### Response
If success, the updated service profile will be return.
```json
{
    "service_id": "<The service profile ID>",
    "service": {
        "type": "<The Type name of the service>",
        "profile": { 
            "_": "// The profile content of the service..."
        }
    }
},
```

### Errors
- If the user or service not exists, an error with status code `404` will be responsed. 
- If the `type` field missmatch the original service, an error with code `400` will be responsed.
- If the data not match the scheme of specific service profile, an error `400` will be responsed.

----------------

## Remove a service from user
`DELETE /service/profile/{uid}/{service_id}`

Only the user with access of `Admin` can delete services profile of any users.

### Request
No request data required.

### Response
- If the service profile not exists, an empty response with status code `204` will return.
- If the service existed and successfully deleted, a response in `200` with the profile deleted in following scheme will return.
```json
{
    "service_id": "<The service profile ID>",
    "service": {
        "type": "<The Type name of the service>",
        "profile": { 
            "_": "// The profile content of the service..."
        }
    }
},
```

### Errors
If the user not exists, an error with status code `404` will be responsed. 