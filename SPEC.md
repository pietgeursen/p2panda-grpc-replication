
---
tags: research
---
# Naive JSON RPC replication protocol

* This protocol assumes that all logs are complete sequences (no entries were deleted or are missing).
* The *log height* is the last known *sequence number* of a *log* for an *author*.

## Concept

* Node `A` wants to find out if Node `B` has any new entries `A` does not know about yet.
* `A` sends a request to `B` which contains what current log heights `A` has per author.

    Example: *"I know that author A has logs with log_id 1, 2, 5 and 8 with these regarding last sequence numbers, I know that author B has log 3 with ..., ..., C has ..."*

    ```
    A 1 12
      2 5
      5 8
      8 55
    B 3 2
      ...
    C ...
    ...
    ```
    
* Node `B` is so kind and already computes the difference between what `A` knows and what `B` knows and returns the delta to `A`.
* Node `A` looks at that response and requests accordingly the missing entries with multiple requests from `B`.
* Node `A` verifies and inserts the missing entries in its database.

## RPC

### `panda_getEntriesDelta`

**Request**

If an empty array is sent the node requests all known entries.

```
[
    {
        "author": "<PUBLIC_KEY>",
        "knownLogHeights": [
            [<LOG_ID>, <SEQUENCE_NUMBER>],
            [..., ...],
            ...
        ]
    },
    {
        ...
    }
]
```

**Response**

The response can be empty if there are no new entries.

```
[
    {
        "author": "<PUBLIC_KEY>",
        "knownLogHeights": [
            [<LOG_ID>, <SEQUENCE_NUMBER>],
            [..., ...],
            ...
        ]
    },
    {
        ...
    }
]
```
    
### `panda_getEntries`

**Request**

When omitting `seqNumBegin` and `seqNumEnd` all entries from the log will be returned. When omitting `seqNumEnd` only one entry will be returned. When omitting `logId` all entries of that author will be returned and any requested sequence number ignored. 

This could be refined.

```rs
enum Request{
  SingleEntry{
    author: PublicKey,
    log_id: u64,
    sequence: u64
  },
  AllLogs{
    author: PublicKey
  }
  Range{
    author: PublicKey,
    log_id: u64,
    sequence_start: u64,
    sequence_end: u64
  }
}

```

```
{
    "author": "<PUBLIC_KEY>",
    "logId": <LOG_ID>,          (optional)
    "seqNumBegin": <SEQ_NUM>,   (optional)
    "seqNumEnd": <SEQ_NUM>      (optional)
}
```

**Response**

```
{
    "entries": [
        {
            "entry_bytes": "<ENTRY_ENCODED>",
            "payload_bytes": "<PAYLOAD_ENCODED>"
        },
        {
            ...
        },
        ...
    ]
}
```
