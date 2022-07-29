### vJoy feeder app for SpaceNavigator 3D mouse

# How to compile for Windows on linux
- ```sudo apt install mingw-w64```
- ```rustup target add x86_64-pc-windows-gnu```
- Add vjoyInterface.lib to ../target/x86_64-pc-windows-gnu/deps/
- ```cargo build --release --target=x86_64-pc-windows-gnu```