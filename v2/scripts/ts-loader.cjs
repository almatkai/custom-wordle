// Validation only. Executes the locked compiler's real JS; no runtime generation fallback.
const fs = require('node:fs');
const vm = require('node:vm');
const ts = require('../fixtures/libraries/typescript/lib/typescript.js');
function load(file, imports, append = '', globals = {}) {
  const source = fs.readFileSync(file, 'utf8') + append;
  const result = ts.transpileModule(source, {
    fileName: file,
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022, jsx: ts.JsxEmit.ReactJSX, esModuleInterop: true },
    reportDiagnostics: true,
  });
  if (result.diagnostics?.some(d => d.category === ts.DiagnosticCategory.Error)) throw new Error('TypeScript transpilation failed');
  const exports = {};
  const context = vm.createContext({ exports, require(name) {
    if (!Object.hasOwn(imports, name)) throw new Error(`Unstubbed import ${name} in ${file}`);
    return imports[name];
  }, ...globals });
  new vm.Script(result.outputText, { filename: file }).runInContext(context, { timeout: 10000 });
  return exports;
}
module.exports = { load, ts };
