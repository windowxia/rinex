Ublox-rnx 
=========

UBlox-rnx is an `UBX` stream to RINEX data converter and server.   
It allows efficient RINEX data production by means of a UBlox GNSS receiver.  

One application expects one receiver device, connected by a serial port.

## File service

By default, the application will generate RINEX files into a local workspace.  
The application can push the files to an FTP server instead, if you deploy with `--ftp=url`.

## Notes on RINEX production

RINEX files span 24h. At midnight, this application will initiate the production
of a new file for the new day of year.  
The day of year is determined when the first position fix is obtained.

## Requirements:

TODO

`libudev`

## Cross compilation

For instance on ARM7 using the Cargo ARM7 configuration 
(I recommend using `rustup` to install the configuration):

```shell
rustup target add armv7-unknown-linux-gnueabihf
cargo build --release \ # release mode: reduce binary size
    --target armv7-unknown-linux-gnueabihf
```

## Getting started

Define the serial port attributes and start formatting Observations.

```shell
ublox-rnx -p /dev/ttyUSB0 -b 9600 -w /tmp/WORKSPACE
```
