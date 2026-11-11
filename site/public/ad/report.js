// Tells the Velmora app whether this ad slot filled, and how tall it is.
//
// The app shows these pages in a sandboxed iframe and can't look inside, so
// the page reports: { source: "velmora-ad", state: "filled" | "empty", height }.
// An empty slot collapses in the app instead of leaving a blank box.
(function () {
  var kind = document.body.getAttribute("data-kind");
  var sent = "";

  function filled() {
    if (kind === "native") {
      var box = document.querySelector("[id^='container-']");
      return !!box && box.children.length > 0 && box.offsetHeight > 20;
    }
    var frame = document.querySelector("iframe");
    return !!frame && frame.offsetHeight > 0;
  }

  function report(state) {
    var height = Math.ceil(document.documentElement.scrollHeight);
    var msg = state + ":" + height;
    if (msg === sent) return;
    sent = msg;
    try {
      parent.postMessage({ source: "velmora-ad", state: state, height: height }, "*");
    } catch (e) {}
  }

  var tries = 0;
  var timer = setInterval(function () {
    tries++;
    if (filled()) {
      report("filled");
    } else if (tries >= 16) {
      // Nothing after 8 seconds: no fill, blocked, or offline.
      clearInterval(timer);
      report("empty");
    }
  }, 500);

  // Native ads change height as their images load.
  if (window.ResizeObserver) {
    new ResizeObserver(function () {
      if (filled()) report("filled");
    }).observe(document.body);
  }
})();
