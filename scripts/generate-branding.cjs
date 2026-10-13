const fs = require('node:fs');
const path = require('node:path');
const { Resvg } = require('../.tools/branding/node_modules/@resvg/resvg-js');

const assets = path.join(__dirname, '..', 'assets');
const source = fs.readFileSync(path.join(assets, 'portal.svg'));
const sizes = [16, 20, 24, 28, 30, 32, 36, 40, 44, 48, 60, 64, 72, 80, 96, 128, 256];
const iconsDirectory = path.join(assets, 'icons');
fs.mkdirSync(iconsDirectory, { recursive: true });
const render = size => new Resvg(source, {
    fitTo: { mode: 'width', value: size },
}).render().asPng();

const images = sizes.map(size => {
    const png = render(size);
    fs.writeFileSync(path.join(iconsDirectory, `portal-${size}.png`), png);
    return png;
});
fs.writeFileSync(path.join(assets, 'portal.png'), render(512));

const directory = Buffer.alloc(6 + sizes.length * 16);
directory.writeUInt16LE(1, 2);
directory.writeUInt16LE(sizes.length, 4);
let offset = directory.length;
sizes.forEach((size, index) => {
    const entry = 6 + index * 16;
    directory[entry] = size === 256 ? 0 : size;
    directory[entry + 1] = directory[entry];
    directory.writeUInt16LE(1, entry + 4);
    directory.writeUInt16LE(32, entry + 6);
    directory.writeUInt32LE(images[index].length, entry + 8);
    directory.writeUInt32LE(offset, entry + 12);
    offset += images[index].length;
});
fs.writeFileSync(path.join(assets, 'portal.ico'), Buffer.concat([directory, ...images]));
console.log(`Generated Portal Nexus branding: ${sizes.join(', ')} px ICO and PNGs, plus 512 px PNG.`);
