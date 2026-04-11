/* @ts-self-types="./rm380z_wasm.d.ts" */

export class Emulator {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        EmulatorFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_emulator_free(ptr, 0);
    }
    /**
     * Store a .COM file for CCP to find.
     * @param {string} name
     * @param {Uint8Array} data
     */
    add_file(name, data) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        wasm.emulator_add_file(this.__wbg_ptr, ptr0, len0, ptr1, len1);
    }
    /**
     * Get the pending Claude prompt (for JS to send via WebSocket).
     * @returns {string}
     */
    claude_get_prompt() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.emulator_claude_get_prompt(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * JS delivers Claude's response. If inject mode, feeds as keystrokes.
     * @param {string} text
     */
    claude_set_response(text) {
        const ptr0 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.emulator_claude_set_response(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {number}
     */
    cursor_col() {
        const ret = wasm.emulator_cursor_col(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    cursor_row() {
        const ret = wasm.emulator_cursor_row(this.__wbg_ptr);
        return ret >>> 0;
    }
    hrg_clear() {
        wasm.emulator_hrg_clear(this.__wbg_ptr);
    }
    /**
     * @param {number} x
     * @param {number} y
     */
    hrg_clear_pixel(x, y) {
        wasm.emulator_hrg_clear_pixel(this.__wbg_ptr, x, y);
    }
    /**
     * @returns {boolean}
     */
    hrg_enabled() {
        const ret = wasm.emulator_hrg_enabled(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {number}
     */
    hrg_height() {
        const ret = wasm.emulator_hrg_height(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {boolean}
     */
    hrg_is_hires() {
        const ret = wasm.emulator_hrg_is_hires(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {number}
     */
    hrg_ptr() {
        const ret = wasm.emulator_hrg_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {boolean} hires
     */
    hrg_set_hires(hires) {
        wasm.emulator_hrg_set_hires(this.__wbg_ptr, hires);
    }
    /**
     * @param {number} x
     * @param {number} y
     */
    hrg_set_pixel(x, y) {
        wasm.emulator_hrg_set_pixel(this.__wbg_ptr, x, y);
    }
    /**
     * @param {boolean} enabled
     */
    hrg_toggle(enabled) {
        wasm.emulator_hrg_toggle(this.__wbg_ptr, enabled);
    }
    /**
     * @returns {number}
     */
    hrg_width() {
        const ret = wasm.emulator_hrg_width(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} offset
     * @param {number} value
     */
    hrg_write(offset, value) {
        wasm.emulator_hrg_write(this.__wbg_ptr, offset, value);
    }
    /**
     * @param {Uint8Array} data
     */
    inject_keys(data) {
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.emulator_inject_keys(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {boolean}
     */
    is_running() {
        const ret = wasm.emulator_is_running(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {number} ch
     */
    key_press(ch) {
        wasm.emulator_key_press(this.__wbg_ptr, ch);
    }
    /**
     * List stored file names.
     * @returns {string[]}
     */
    list_files() {
        const ret = wasm.emulator_list_files(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @param {Uint8Array} data
     */
    load_com(data) {
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.emulator_load_com(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Load a .COM by name from stored files. Returns true if found.
     * @param {string} name
     * @returns {boolean}
     */
    load_com_by_name(name) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.emulator_load_com_by_name(this.__wbg_ptr, ptr0, len0);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    needs_claude() {
        const ret = wasm.emulator_needs_claude(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    needs_key() {
        const ret = wasm.emulator_needs_key(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    needs_net() {
        const ret = wasm.emulator_needs_net(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {number} conn_id
     * @returns {string}
     */
    net_get_request(conn_id) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.emulator_net_get_request(this.__wbg_ptr, conn_id);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {number} conn_id
     * @returns {Uint8Array}
     */
    net_get_request_body(conn_id) {
        const ret = wasm.emulator_net_get_request_body(this.__wbg_ptr, conn_id);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @param {number} conn_id
     * @returns {boolean}
     */
    net_is_ws(conn_id) {
        const ret = wasm.emulator_net_is_ws(this.__wbg_ptr, conn_id);
        return ret !== 0;
    }
    /**
     * @param {number} drive
     */
    net_mount(drive) {
        wasm.emulator_net_mount(this.__wbg_ptr, drive);
    }
    /**
     * @param {number} conn_id
     * @param {Uint8Array} data
     */
    net_set_response(conn_id, data) {
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.emulator_net_set_response(this.__wbg_ptr, conn_id, ptr0, len0);
    }
    /**
     * @param {number} conn_id
     * @param {Uint8Array} data
     */
    net_ws_receive(conn_id, data) {
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.emulator_net_ws_receive(this.__wbg_ptr, conn_id, ptr0, len0);
    }
    /**
     * @param {number} conn_id
     * @returns {Uint8Array}
     */
    net_ws_take_send(conn_id) {
        const ret = wasm.emulator_net_ws_take_send(this.__wbg_ptr, conn_id);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    constructor() {
        const ret = wasm.emulator_new();
        this.__wbg_ptr = ret >>> 0;
        EmulatorFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {number} max_steps
     * @returns {number}
     */
    run(max_steps) {
        const ret = wasm.emulator_run(this.__wbg_ptr, max_steps);
        return ret >>> 0;
    }
    /**
     * Write a string to the VDU.
     * @param {string} s
     */
    vdu_print(s) {
        const ptr0 = passStringToWasm0(s, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.emulator_vdu_print(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {number}
     */
    vdu_ptr() {
        const ret = wasm.emulator_vdu_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Write a character to the VDU (for JS-side CCP echo).
     * @param {number} ch
     */
    vdu_write(ch) {
        wasm.emulator_vdu_write(this.__wbg_ptr, ch);
    }
    /**
     * @returns {number}
     */
    waiting_net_id() {
        const ret = wasm.emulator_waiting_net_id(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) Emulator.prototype[Symbol.dispose] = Emulator.prototype.free;
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_throw_6b64449b9b9ed33c: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbindgen_cast_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./rm380z_wasm_bg.js": import0,
    };
}

const EmulatorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_emulator_free(ptr >>> 0, 1));

function getArrayJsValueFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getDataViewMemory0();
    const result = [];
    for (let i = ptr; i < ptr + 4 * len; i += 4) {
        result.push(wasm.__wbindgen_externrefs.get(mem.getUint32(i, true)));
    }
    wasm.__externref_drop_slice(ptr, len);
    return result;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('rm380z_wasm_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
