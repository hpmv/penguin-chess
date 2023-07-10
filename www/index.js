const worker = new Worker(new URL("./worker.js", import.meta.url));

window.doSearch = function doSearch() {
    const stop = new SharedArrayBuffer(1);
    worker.postMessage({
        search: [0, 1, 3, 4, 20, 21, 23, 24, 22, 2, 1],
        stop,
    });

    setTimeout(() => {
        new Uint8Array(stop)[0] = 1;
    }, 2000);
}

worker.onmessage = (msg) => {
    console.log(msg);
};
