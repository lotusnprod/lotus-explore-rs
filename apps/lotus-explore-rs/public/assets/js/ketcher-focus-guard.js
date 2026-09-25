(function () {
  var originalFocus = HTMLElement.prototype.focus;
  var blocked = false;
  HTMLElement.prototype.focus = function () {
    if (!blocked && this.matches && this.matches("[data-cliparea]")) {
      blocked = true;
      return;
    }
    return originalFocus.apply(this, arguments);
  };
})();
