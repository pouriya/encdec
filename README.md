# EncDec
A terminal encryption utility that supports multipart and compression.

## Installation
Download the latest version:
* **GNU/Linux** (Built on Ubuntu):
    * Musl (Statically linked):       [**download**](https://github.com/pouriya/encdec/releases/download/latest/encdec-latest-x86_64-unknown-linux-musl-ubuntu-22.04)
    * GNU (Dynamic linking to glibc): [**download**](https://github.com/pouriya/encdec/releases/download/latest/encdec-latest-x86_64-unknown-linux-gnu-ubuntu-22.04)
* **macOS**:
    * v11: [**download**](https://github.com/pouriya/encdec/releases/download/latest/encdec-latest-x86_64-apple-darwin-macos-11)
    * v12: [**download**](https://github.com/pouriya/encdec/releases/download/latest/encdec-latest-x86_64-apple-darwin-macos-12)
    * v13: [**download**](https://github.com/pouriya/encdec/releases/download/latest/encdec-latest-x86_64-apple-darwin-macos-13)
* **Windows**:
    * v2019:
        * MSVC: [**download**](https://github.com/pouriya/encdec/releases/download/latest/encdec-latest-x86_64-pc-windows-msvc-windows-2019.exe)
        * GNU:  [**download**](https://github.com/pouriya/encdec/releases/download/latest/encdec-latest-x86_64-pc-windows-gnu-windows-2019.exe)
    * latest (v2022):
      * MSVC: [**download**](https://github.com/pouriya/encdec/releases/download/latest/encdec-latest-x86_64-pc-windows-msvc-windows-2022.exe)
      * GNU:  [**download**](https://github.com/pouriya/encdec/releases/download/latest/encdec-latest-x86_64-pc-windows-gnu-windows-2022.exe)

## Usage
```shell
encdec --help
```
```text
A terminal encryption utility that supports multipart and compression.

Usage: encdec <COMMAND>

Commands:
  gen   Generates private & public PEM files
  enc   Encryption
  dec   Decryption
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Generate key pairs
#### Usage
```shell
encdec gen --help
```
```text
Generates private & public PEM files

Usage: encdec gen [OPTIONS] --name <NAME>

Options:
  -o, --output-directory <OUTPUT_DIRECTORY>
          Output directory to save keys [default: /p/encdec]
  -n, --name <NAME>
          Name of the generated keys that transforms to <NAME>.PRIV.pem and <NAME>.PUB.pem
  -k, --key-size-in-bytes <KEY_SIZE_IN_BYTES>
          Private key modulus key size in bytes [default: 256] [possible values: 128, 256, 512]
  -h, --help
          Print help
```
#### Example
```shell
encdec gen -n test
```
```text
Attempt to generate RSA private key with bit size 2048.
Generated private key
Attempt to generate RSA public key
Generated public key
Saved private PEM contents in "/p/encdec/test.PRIV.pem"
Saved private PEM contents in "/p/encdec/test.PUB.pem"
```
```shell
ls test*
```
```text
test.PRIV.pem  test.PUB.pem
```
```shell
head -n 3 test.PRIV.pem # read first 3 lines
```
```text
-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEAsxUmO3BcU0aM6uaP5ctNq4FtdbC+qX7cWfaL727Mt5N+uoLP
OXL2h65i9s67Dn0ZQNiROIYVcYSYPSxHoNhng9g60w5oesujZvGFvJGIwI5MfVGg
```
```shell
head -n 3 test.PUB.pem
```
```text
-----BEGIN RSA PUBLIC KEY-----
MIIBCgKCAQEAsxUmO3BcU0aM6uaP5ctNq4FtdbC+qX7cWfaL727Mt5N+uoLPOXL2
h65i9s67Dn0ZQNiROIYVcYSYPSxHoNhng9g60w5oesujZvGFvJGIwI5MfVGgnCes
```

### Encryption
#### Usage
```shell
encdec enc --help
```
```text
Encryption

Usage: encdec enc [OPTIONS] --public-pem-file <PUBLIC_PEM_FILE> --input-filename <INPUT_FILENAME> --output-filename <OUTPUT_FILENAME>

Options:
  -p, --public-pem-file <PUBLIC_PEM_FILE>
          Public key PEM file
  -i, --input-filename <INPUT_FILENAME>
          Input file to encrypt
  -o, --output-filename <OUTPUT_FILENAME>
          Output file to save encrypted contents
  -z, --zip
          If input is too large, encrypts input part by part and stores them inside a .zip file
  -h, --help
          Print help
```
#### Example
```shell
cat secret.txt # We are going to encrypt this sample
```
```text
Lorem ipsum dolor sit amet, consectetur adipiscing elit. In tellus elit, tristique vitae mattis egestas, ultricies vitae risus. Quisque sit amet quam ut urna aliquet
molestie. Proin blandit ornare dui, a tempor nisl accumsan in. Praesent a consequat felis. Morbi metus diam, auctor in auctor vel, feugiat id odio. Curabitur ex ex,
dictum quis auctor quis, suscipit id lorem. Aliquam vestibulum dolor nec enim vehicula, porta tristique augue tincidunt. Vivamus ut gravida est. Sed pellentesque, dolor
vitae tristique consectetur, neque lectus pulvinar dui, sed feugiat purus diam id lectus. Class aptent taciti sociosqu ad litora torquent per conubia nostra, per
inceptos himenaeos. Maecenas feugiat velit in ex ultrices scelerisque id id neque.
```
```shell
encdec enc -p test.PUB.pem -i secret.txt -o encrypted
```
```text
Attempt to check for PKCS#1 method
Updated chunk size to 128
Saved encrypted output to "encrypted.zip"
```
```shell
unzip -l encrypted.zip # List files inside .zip file
```
```text
Archive:  encrypted.zip
  Length      Date    Time    Name
---------  ---------- -----   ----
        0  2024-04-09 21:45   parts/
      256  2024-04-09 21:45   parts/encrypted.1
      256  2024-04-09 21:45   parts/encrypted.2
      256  2024-04-09 21:45   parts/encrypted.3
      256  2024-04-09 21:45   parts/encrypted.4
      256  2024-04-09 21:45   parts/encrypted.5
      256  2024-04-09 21:45   parts/encrypted.6
---------                     -------
     1536                     7 files
```

### Decryption
#### Usage
```shell
encdec dec --help
```
```text
Decryption

Usage: encdec dec --private-pem-file <PRIVATE_PEM_FILE> --input-filename <INPUT_FILENAME> --output-filename <OUTPUT_FILENAME>

Options:
  -p, --private-pem-file <PRIVATE_PEM_FILE>  Private key PEM file
  -i, --input-filename <INPUT_FILENAME>      Input file to decrypt
  -o, --output-filename <OUTPUT_FILENAME>    Output file to save decrypted contents
  -h, --help                                 Print help
```
#### Example
```shell
encdec dec -p test.PRIV.pem -i encrypted.zip -o decrypted-secret.txt
```
```text
Attempt to check for PKCS#1 method
Saved decrypted output to "decrypted-secret.txt"
```
```shell
cat decrypted-secret.txt
```
```text
Lorem ipsum dolor sit amet, consectetur adipiscing elit. In tellus elit, tristique vitae mattis egestas, ultricies vitae risus. Quisque sit amet quam ut urna aliquet
molestie. Proin blandit ornare dui, a tempor nisl accumsan in. Praesent a consequat felis. Morbi metus diam, auctor in auctor vel, feugiat id odio. Curabitur ex ex,
dictum quis auctor quis, suscipit id lorem. Aliquam vestibulum dolor nec enim vehicula, porta tristique augue tincidunt. Vivamus ut gravida est. Sed pellentesque, dolor
vitae tristique consectetur, neque lectus pulvinar dui, sed feugiat purus diam id lectus. Class aptent taciti sociosqu ad litora torquent per conubia nostra, per
inceptos himenaeos. Maecenas feugiat velit in ex ultrices scelerisque id id neque.
```
```shell
md5sum secret.txt decrypted-secret.txt 
```
```text
e17df2471b7b1037c63ca60723f56457  secret.txt
e17df2471b7b1037c63ca60723f56457  decrypted-secret.txt
```


# To contributors
I ❤️ PR from everyone and I appreciate your help but before opening a PR, file an issue and describe your feature, bug, etc.