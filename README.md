# Mufin
***Bluetooth based IoT connection library***

## Building mufin
### Requirements
- Rust and Cargo
- Nodejs and Npm

### Methods
1. use `npm install` command to install the napi cli
2. use `npm run build` and there will be a `index.node` file in the root of the project
3. use the `index.node` file in nodejs
```js
let addon = require('./index.node')
addon.bluetooth()
```

