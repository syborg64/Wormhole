# make virtuals
rm -rf virtual*

mkdir virtual
mkdir virtual2
mkdir virtual3

mkdir virtual/subdir
mkdir virtual2/subdir
mkdir virtual3/subdir

echo "content" > virtual/file
echo "content" > virtual2/file
echo "content" > virtual3/file

echo "subfile content" > virtual/subdir/subfile
echo "subfile content" > virtual2/subdir/subfile
echo "subfile content" > virtual3/subdir/subfile

(cargo run --bin wormhole-cli -- template -C virtual) > /dev/null 2>&1
(cargo run --bin wormhole-cli -- template -C virtual2) > /dev/null 2>&1
(cargo run --bin wormhole-cli -- template -C virtual3) > /dev/null 2>&1

tree -a virtual*/
