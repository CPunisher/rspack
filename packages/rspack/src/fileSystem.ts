import {
	JsDirent,
	JsMetadata,
	ThreadsafeInputNodeFS,
	ThreadsafeOutputNodeFS
} from "@rspack/binding";
import util from "util";

import {
	InputFileSystem,
	join,
	mkdirp,
	OutputFileSystem,
	rmrf
} from "./util/fs";
import { memoizeFn } from "./util/memoize";

const NOOP_FILESYSTEM: ThreadsafeOutputNodeFS = {
	writeFile() {},
	removeFile() {},
	mkdir() {},
	mkdirp() {},
	removeDirAll() {}
};

class ThreadsafeWritableNodeFS implements ThreadsafeOutputNodeFS {
	writeFile!: (name: string, content: Buffer) => Promise<void> | void;
	removeFile!: (name: string) => Promise<void> | void;
	mkdir!: (name: string) => Promise<void> | void;
	mkdirp!: (name: string) => Promise<string | void> | string | void;
	removeDirAll!: (name: string) => Promise<string | void> | string | void;

	constructor(fs: OutputFileSystem | null) {
		if (!fs) {
			// This happens when located in a child compiler.
			Object.assign(this, NOOP_FILESYSTEM);
			return;
		}
		this.writeFile = memoizeFn(() => util.promisify(fs.writeFile.bind(fs)));
		this.removeFile = memoizeFn(() => util.promisify(fs.unlink.bind(fs)));
		this.mkdir = memoizeFn(() => util.promisify(fs.mkdir.bind(fs)));
		this.mkdirp = memoizeFn(() => util.promisify(mkdirp.bind(null, fs)));
		this.removeDirAll = memoizeFn(() => util.promisify(rmrf.bind(null, fs)));
	}

	static __into_binding(fs: OutputFileSystem | null) {
		return new this(fs);
	}
}

class ThreadsafeReadableNodeFS implements ThreadsafeInputNodeFS {
	readFile: (
		name: string
	) => string | Buffer | void | Promise<string | Buffer | void>;
	readDir: (name: string) => void | JsDirent[] | Promise<void | JsDirent[]>;
	stat: (name: string) => void | JsMetadata | Promise<void | JsMetadata>;
	lstat: (name: string) => void | JsMetadata | Promise<void | JsMetadata>;
	realpath: (name: string) => string | void | Promise<string | void>;

	constructor(fs: InputFileSystem | null) {
		if (!fs) {
			throw new Error("ThreadsafeReadableNodeFS requires an InputFileSystem");
		}

		this.readFile = memoizeFn(() => util.promisify(fs.readFile.bind(fs)));
		this.readDir = memoizeFn(() =>
			util.promisify<string, JsDirent[] | void>((arg0, callback) => {
				fs.readdir(arg0, (err, files) => {
					if (err) {
						callback(err);
						return;
					}
					const dirents: JsDirent[] = [];
					let count = files!.length;
					for (const file of files!) {
						const joinPath = join(fs, arg0, file);
						fs.stat(joinPath, (err, stat) => {
							if (err) {
								callback(err);
								return;
							}
							dirents.push({
								path: joinPath,
								metadata: {
									isDir: stat!.isDirectory(),
									isFile: stat!.isFile(),
									isSymlink: stat!.isSymbolicLink()
								}
							});
							if (--count === 0) {
								callback(null, dirents);
							}
						});
					}
				});
			})
		);
		this.stat = memoizeFn(() =>
			util.promisify<string, JsMetadata | void>((arg0, callback) => {
				fs.stat(arg0, (err, stat) => {
					if (err) {
						callback(err);
						return;
					}
					callback(null, {
						isDir: stat!.isDirectory(),
						isFile: stat!.isFile(),
						isSymlink: stat!.isSymbolicLink()
					});
				});
			})
		);
		this.lstat = memoizeFn(() =>
			util.promisify<string, JsMetadata | void>((arg0, callback) => {
				fs.lstat!(arg0, (err, stat) => {
					if (err) {
						callback(err);
						return;
					}
					callback(null, {
						isDir: stat!.isDirectory(),
						isFile: stat!.isFile(),
						isSymlink: stat!.isSymbolicLink()
					});
				});
			})
		);
		this.realpath = memoizeFn(() => util.promisify(fs.realpath!.bind(fs)));
	}

	static __into_binding(fs: InputFileSystem | null) {
		return new this(fs);
	}
}

export { ThreadsafeReadableNodeFS, ThreadsafeWritableNodeFS };
