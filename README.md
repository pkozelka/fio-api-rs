# Fio Client

This library aims to fully support the FIO Banking API, offering both low-level access to the entire functionality, 
and higher-level access for more convenient work in typical scenarios. 

## Rust APIs

### Low-level API

State: mostly implemented

Low-level API is, essentially, just a kit for preparing requests and processing responses. 
There is very simple type-based support, and little or no additional logic.

This allows the implementation to be quite easily adjusted when FIO API specification changes.

### Higher-level API

State: not implemented yet

This helps the caller to not have to think about certain limitations, like maximum API call rate, data formats etc.

The plan is to implement following features:

- [ ] track token usage time to prevent failure
- [ ] work with multiple RO/RW tokens to minimize waiting for next API call time
- [ ] expose fully-parsing functionality, internally using any convenient format (probably CSV)

