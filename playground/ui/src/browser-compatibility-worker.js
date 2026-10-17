onmessage = async () => postMessage({
  languages: [...navigator.languages],
  userAgent: navigator.userAgent,
  hints: navigator.userAgentData ? await navigator.userAgentData.getHighEntropyValues(['fullVersionList']) : null,
});
