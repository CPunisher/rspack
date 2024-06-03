const rspack = require("@rspack/core");
const { Volume, createFsFromVolume } = require("memfs");

function installFS(compiler, files) {
  const val = Volume.fromJSON(files);
  const fs = createFsFromVolume(val);
  compiler.inputFileSystem = fs;
  compiler.intermediateFileSystem = fs;
  compiler.outputFileSystem = fs;
  return val;
}

describe("Basic Compilation", () => {
	it("should compile js", () => {
    const compiler = rspack({
      entry: "/index.js",
      output: {
        filename: "/output.js",
      },
    });
    const val = installFS(compiler, {
      "/index.js": `
      const a = 1;
      console.log(a);
    `,
    });

    compiler.run(() => {
      const files = val.toJSON();
      expect(files["/output.js"]).toBe("console.log(1);")
    });
  });

  it("should compile ts", () => {
    const compiler = rspack({
      entry: "/index.ts",
      output: {
        filename: "/output.js",
      },
      resolve: {
        extensions: ["...", ".ts"]
      },
      module: {
        rules: [
          {
            test: /\.ts$/,
            use: [
              {
                loader: "builtin:swc-loader",
                options: {
                  jsc: {
                    parser: {
                      syntax: "typescript"
                    }
                  }
                }
              }
            ],
            type: "javascript/auto"
          }
        ]
      }
    });
    const val = installFS(compiler, {
      "/index.ts": `
      const a: number = 1;
      console.log(a);
    `,
  });

  compiler.run(() => {
    const files = val.toJSON();
    expect(files["/output.js"]).toBe("console.log(1);")
  });
  });
});
