"""Strip staged native images and retain matching private symbol artifacts."""
import hashlib
import json
import shutil
import subprocess
import sys

MACH_MAGICS = {b'\xcf\xfa\xed\xfe', b'\xce\xfa\xed\xfe', b'\xfe\xed\xfa\xcf', b'\xfe\xed\xfa\xce', b'\xca\xfe\xba\xbe', b'\xbe\xba\xfe\xca'}


def native_image(path):
    with path.open('rb') as stream:
        magic = stream.read(4)
    return magic in MACH_MAGICS or magic == b'\x7fELF' or magic[:2] == b'MZ'


def process(root, *, strip=True, symbols=None, metadata=None, source_root=None):
    """Only modify staging copies; export originals before stripping/signing."""
    records = []
    if symbols:
        symbols.mkdir(parents=True, exist_ok=False)
    before = after = 0
    exported = {}
    for path in sorted(p for p in root.rglob('*') if p.is_file()):
        if not native_image(path):
            continue
        relative = path.relative_to(root)
        with path.open('rb') as stream:
            original_hash = hashlib.file_digest(stream, 'sha256').hexdigest()
        original_size = path.stat().st_size
        before += original_size
        record = {'path': relative.as_posix(), 'original_sha256': original_hash}
        if symbols:
            original = symbols / exported[original_hash] if original_hash in exported else symbols / 'binaries' / original_hash / path.name
            record['binary'] = original.relative_to(symbols).as_posix()
            original.parent.mkdir(parents=True, exist_ok=True)
            if original_hash not in exported:
                shutil.copy2(path, original)
                source = source_root / path.name if source_root else path
                for pdb in (source.with_suffix('.pdb'), source.with_name(source.name + '.pdb')):
                    if pdb.is_file():
                        shutil.copy2(pdb, original.parent / pdb.name)
        if sys.platform == 'darwin':
            uuid = subprocess.run(['xcrun', 'dwarfdump', '--uuid', str(path)], check=True, capture_output=True, text=True)
            record['uuids'] = [line.split()[1] for line in uuid.stdout.splitlines() if line.startswith('UUID:')]
            if symbols and original_hash not in exported:
                source = source_root / path.name if source_root else path
                subprocess.run(['xcrun', 'dsymutil', str(source), '-o', str(original) + '.dSYM'], check=True, cwd=source.parent)
            if strip:
                subprocess.run(['xcrun', 'strip', '-S', '-x', str(path)], check=True)
                subprocess.run(['codesign', '--force', '--sign', '-', str(path)], check=True)
        elif sys.platform.startswith('linux') and strip:
            subprocess.run(['strip', '--strip-debug', '--strip-unneeded', str(path)], check=True)
        # MSVC debug information is external: preserve the image/export table.
        after += path.stat().st_size
        with path.open('rb') as stream:
            record['runtime_sha256'] = hashlib.file_digest(stream, 'sha256').hexdigest()
        if symbols:
            exported[original_hash] = original.relative_to(symbols).as_posix()
        records.append(record)
    if symbols:
        (symbols / 'symbols.json').write_text(json.dumps({'schema_version': 1, **(metadata or {}), 'images': records}, indent=2) + '\n', encoding='utf-8')
    print(f'native_bytes_before={before}\nnative_bytes_after={after}\nnative_bytes_saved={before - after}')
    return records
