# JSON HTTP API 


## `POST /transaction`

### Coin transaction

Request

```json
{
    "recipient": "<public_key>",
    "amount": 123,
}
```

### Message transaction

Request

```json
{
    "recipient": "<public_key>",
    "message": "hello",
}
```

## `POST /stake`

Request

```json
{
    "amount": 123
}
```

## `GET /block`

Request

```json
{
    hash: <block_hash>,
    signature: <block_signature>,
    data: {
        "timestamp": DateTime<Utc>,
        "transactions": [
            {
                hash: <tx_hash>,
                signature: <signature>,
                data: {
                    sender_address: <public_key>,
                    kind: {
                        "type": "Coin",
                        "amount": 123,
                        "recipient": <public_key>,
                    },
                    nonce: 123,
                }
            },
            {
                hash: <tx_hash>,
                signature: <signature>,
                data: {
                    sender_address: <public_key>,
                    kind: {
                        "type": "Message",
                        "message": "hello",
                        "recipient": <public_key>,
                    },
                    nonce: 123,
                }
            },
            {
                hash: <tx_hash>,
                signature: <signature>,
                data: {
                    sender_address: <public_key>,
                    kind: {
                        "type": "Stake",
                        "amount": 123
                    },
                    nonce: 123,
                }
            },
        ],
        "validator": <public_key>,
        "parent_hash": <hash>,
    }
}
```

## `GET /balance`

Response

```json
{
    balance: 123,
    stake: 12,
}
```

## `GET /info`

Response

```json
{
    name: "node-1",
    public_key: <node_public_key>,
    capacity: 5,
}
```
