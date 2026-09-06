// Theme: remembered per browser, follows the system the first time.
(function () {
  var root = document.documentElement;
  var stored = null;
  try { stored = localStorage.getItem("fyf-theme"); } catch (e) {}
  var system = window.matchMedia && window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
  root.setAttribute("data-theme", stored || system);
  document.addEventListener("DOMContentLoaded", function () {
    var b = document.getElementById("theme-toggle");
    if (!b) return;
    function label() { b.textContent = root.getAttribute("data-theme") === "light" ? b.getAttribute("data-dark") : b.getAttribute("data-light"); }
    label();
    b.addEventListener("click", function () {
      var next = root.getAttribute("data-theme") === "light" ? "dark" : "light";
      root.setAttribute("data-theme", next);
      try { localStorage.setItem("fyf-theme", next); } catch (e) {}
      label();
    });
  });
})();
