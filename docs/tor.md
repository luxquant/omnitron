### MacOS

```sh
brew install automake
brew install libevent
brew install openssl
./configure --disable-manpage --disable-html-manual --disable-asciidoc --disable-system-torrc --disable-unittests --enable-static-zlib --enable-static-tor --with-libevent-dir=$(brew --prefix libevent) --enable-static-openssl --with-openssl-dir=$(brew --prefix openssl) --with-zlib-dir=$(brew --prefix zlib)
xcode-select -p
```
