import("../pkg/index.js").then(module =>{
    let mod = new module.WebHandle();
    mod.start("main_canvas")
 });