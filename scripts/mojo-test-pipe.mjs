// Test-only in-memory Mojo core handles. All message construction, headers,
// request IDs and response correlation in clients use Chromium's real runtime.
export function installMojo(onMessage) {
  class Handle {
    queue = [];
    callback = null;
    closed = false;
    watch(_signals,callback) { this.callback=callback; return {cancel:()=>this.callback=null}; }
    readMessage() {
      if(this.queue.length) return {result:0,...this.queue.shift()};
      return {result:this.peer.closed?9:17};
    }
    writeMessage(buffer,handles) {
      if(this.peer.closed) return 9;
      this.peer.queue.push({buffer,handles});
      queueMicrotask(()=>this.peer.callback?.(0));
      return 0;
    }
    close() { this.closed=true;queueMicrotask(()=>this.peer.callback?.(9)); }
  }
  globalThis.Mojo = {
    RESULT_OK:0,RESULT_SHOULD_WAIT:17,RESULT_FAILED_PRECONDITION:9,
    createMessagePipe() {
      const handle0=new Handle(),handle1=new Handle();
      handle0.peer=handle1;handle1.peer=handle0;
      return {handle0,handle1};
    },
    bindInterface(name,handle) {
      handle.watch({readable:true},()=>{
        for(let read;(read=handle.readMessage()).result===0;) onMessage(name,read.buffer,handle);
      });
    },
  };
}
export function replyTo(request,payload) {
  const input=new DataView(request);
  const buffer=new ArrayBuffer(32+payload.length);
  const view=new DataView(buffer);
  view.setUint32(0,32,true);view.setUint32(4,1,true);
  view.setUint32(12,input.getUint32(12,true),true);
  view.setUint32(16,2,true);
  view.setBigUint64(24,input.getBigUint64(24,true),true);
  new Uint8Array(buffer,32).set(payload);
  return buffer;
}
