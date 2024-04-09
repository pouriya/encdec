#! /bin/sh

ENCDEC="$1"

set -xe

rm -rf tmp && mkdir tmp && cd tmp
openssl genrsa -out private.pem 1024
openssl rsa -in private.pem -pubout > public.pem
echo "B1g S3cr3t" > msg.txt
openssl rsautl -encrypt -inkey public.pem -pubin -in msg.txt -out msg.openssl.enc
${ENCDEC} enc -p public.pem -i msg.txt -o msg.encdec.enc
ls -lash msg.openssl.enc msg.encdec.enc
openssl rsautl -decrypt -inkey private.pem -in msg.openssl.enc -out msg.openssl.txt
${ENCDEC} dec -p private.pem -i msg.encdec.enc -o msg.encdec.txt
cat msg.openssl.txt
cat msg.encdec.txt
rm -rf msg.openssl.txt msg.encdec.txt
openssl rsautl -decrypt -inkey private.pem -in msg.encdec.enc -out msg.openssl.txt
${ENCDEC} dec -p private.pem -i msg.openssl.enc -o msg.encdec.txt
cat msg.openssl.txt
cat msg.encdec.txt

cd - 