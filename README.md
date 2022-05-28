# Mufin
***Bluetooth based IoT connection library***

## Using mufin
### Requirements
- Rust and Cargo
- Nodejs and Npm

### Methods
1. Install mufin with `npm install @narwhallab/mufin`
2. use `@narwhallab/mufin` in nodejs
```ts
import mufin from '@narwhallab/mufin'

mufin.scanBluetooth().then(() => {
    mufin.connectBluetooth("<btaddr>").then(async () => {
        await mufin.writeBluetooth("<btaddr>", "My Message");
        await mufin.disconnectBluetooth("<btaddr>")
    })
})
```

## TODO
- Reading from bluetooth