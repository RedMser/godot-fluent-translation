const fs = require('fs');
const archiver = require('archiver');
const { basename, join } = require('path');

const versions = ['forked', 'default'];
const platforms = ['windows', 'linux'];
const targets = ['debug', 'release'];

function getSingleFile(directory) {
    const files = fs.readdirSync(directory);
    if (files.length !== 1) {
        console.log(files);
        throw new Error('Expected a single file, but got ' + files.length);
    }
    return files[0];
}

for (const version of versions) {
    for (const platform of platforms) {
        const zipStream = fs.createWriteStream(`./${version}.${platform}.godot_fluent_translation.zip`);
        const zip = archiver('zip', { zlib: { level: 9 }});
        zip.on('warning', (err) => {
            console.warn("Warning:", err);
        });
        zip.on('error', (err) => {
            console.error("Error:", err);
        });
        zip.pipe(zipStream);

        const addFile = (sourcePath, targetPath = "") => {
            if (targetPath === '') {
                targetPath = sourcePath;
            }
            console.log("[Add File]", sourcePath, "->", targetPath);
            zip.append(fs.createReadStream(`./${sourcePath}`), { name: `addons/godot-fluent-translation/${targetPath}` });
        };

        addFile('LICENSE');
        addFile('README.md');
        addFile('godot-fluent-translation.gdextension');

        for (const target of targets) {
            const libraryDir = `${version}.${platform}.godot_fluent_translation.${target}`;
            const libraryFile = basename(getSingleFile(`./${libraryDir}/`));
            addFile(join(libraryDir, libraryFile), `${target}/${libraryFile}`);
        }

        zip.finalize();
    }
}
