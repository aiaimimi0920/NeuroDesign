/* Offline primitives shared by both themes. No network, storage, or product APIs. */
(() => {
  "use strict";
  const paths = {
    mark: '<path d="M3 13V3l10 10V3"/><path d="M3 13h10"/>',
    grid: '<rect x="2.5" y="2.5" width="4" height="4"/><rect x="9.5" y="2.5" width="4" height="4"/><rect x="2.5" y="9.5" width="4" height="4"/><rect x="9.5" y="9.5" width="4" height="4"/>',
    list: '<path d="M6 4h8M6 8h8M6 12h8M2 4h1M2 8h1M2 12h1"/>',
    plus: '<path d="M8 3v10M3 8h10"/>',
    minus: '<path d="M3 8h10"/>',
    search: '<circle cx="7" cy="7" r="4.5"/><path d="m10.5 10.5 3 3"/>',
    arrow: '<path d="M3 8h10M9 4l4 4-4 4"/>',
    chevron: '<path d="m4 6 4 4 4-4"/>',
    close: '<path d="m4 4 8 8M12 4l-8 8"/>',
    check: '<path d="m3 8 3 3 7-7"/>',
    info: '<circle cx="8" cy="8" r="6"/><path d="M8 7v4M8 4.5v.2"/>',
    warning: '<path d="m8 2 6.5 12h-13L8 2Z"/><path d="M8 6v3M8 11v.2"/>',
    moon: '<path d="M13.5 9.5A6 6 0 0 1 6.5 2a6 6 0 1 0 7 7.5Z"/>',
    sun: '<circle cx="8" cy="8" r="3"/><path d="M8 1v1M8 14v1M1 8h1M14 8h1M3 3l1 1M12 12l1 1M3 13l1-1M12 4l1-1"/>',
    folder: '<path d="M2 4h5l1.5 2H14v7H2V4Z"/><path d="M2 6h6"/>',
    file: '<path d="M4 2h5l3 3v9H4V2Z"/><path d="M9 2v4h3M6 9h4M6 11h3"/>',
    image: '<rect x="2" y="2" width="12" height="12"/><circle cx="5.5" cy="5.5" r="1"/><path d="m3 12 4-4 3 3 2-2 2 2"/>',
    code: '<path d="m5 4-4 4 4 4M11 4l4 4-4 4M9 2 7 14"/>',
    download: '<path d="M8 2v8M5 7l3 3 3-3M3 11v3h10v-3"/>',
    settings: '<path d="M2 4h12M2 8h12M2 12h12"/><path d="M5 2v4M11 6v4M6 10v4"/>',
    star: '<path d="m8 1.8 1.8 3.8 4.2.6-3 3 .7 4.2L8 11.5l-3.7 1.9.7-4.2-3-3 4.2-.6L8 1.8Z"/>',
    trash: '<path d="M2 4h12M6 4V2h4v2M4 4l1 10h6l1-10M7 6v5M9 6v5"/>',
    copy: '<path d="M6 6h8v8H6zM3 10H2V2h8v1"/>',
    edit: '<path d="m3 10 7-7 3 3-7 7-4 1 1-4Z"/><path d="m9 4 3 3"/>',
    menu: '<path d="M2 4h12M2 8h12M2 12h12"/>',
    more: '<circle cx="3" cy="8" r=".7"/><circle cx="8" cy="8" r=".7"/><circle cx="13" cy="8" r=".7"/>',
    clock: '<circle cx="8" cy="8" r="6"/><path d="M8 4v4l3 2"/>',
    eye: '<path d="M1 8s2.5-5 7-5 7 5 7 5-2.5 5-7 5-7-5-7-5Z"/><circle cx="8" cy="8" r="2"/>',
    lock: '<rect x="3" y="7" width="10" height="7"/><path d="M5 7V5a3 3 0 0 1 6 0v2M8 10v1"/>',
    pin: '<path d="m6 2 8 8-3 1-2 3-2-5-5-2 3-2 1-3Z"/><path d="m6 10-4 4"/>'
  };
  const D = window.NeuroDemo = {
    theme: document.body.dataset.theme,
    compact: document.body.dataset.theme === "neuro-mini",
    icon(name) {
      return '<svg class="icon" viewBox="0 0 16 16" aria-hidden="true">' + (paths[name] || paths.file) + '</svg>';
    },
    escape(value) {
      return String(value).replace(/[&<>"']/g, (character) => ({
        "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;"
      })[character]);
    },
    previewBar(page) {
      return '<a class="skip-link" href="#main-content">Skip to content</a>' +
        '<header class="preview-bar"><div class="row">' +
        '<a class="brand-link" href="../../neuro.html">Neuro themes</a>' +
        '<span>' + this.theme + '</span>' +
        '<span class="preview-note muted">Demo · resets on refresh</span></div>' +
        '<nav class="row" aria-label="Preview navigation">' +
        '<a href="' + (page === "gallery" ? "example.html" : "index.html") + '">' + (page === "gallery" ? "Website example ↗" : "Component atlas ↗") + '</a>' +
        '<a href="DESIGN.zh-CN.md">中文规范</a>' +
        '<button class="btn ghost" id="mode-toggle" type="button" aria-label="Switch to light mode">' + this.icon("sun") + 'Light</button></nav></header>';
    },
    setup() {
      document.getElementById("mode-toggle").addEventListener("click", () => {
        const light = document.documentElement.dataset.mode !== "light";
        document.documentElement.dataset.mode = light ? "light" : "dark";
        const button = document.getElementById("mode-toggle");
        button.innerHTML = this.icon(light ? "moon" : "sun") + (light ? "Dark" : "Light");
        button.setAttribute("aria-label", "Switch to " + (light ? "dark" : "light") + " mode");
      });
      document.addEventListener("click", (event) => {
        document.querySelectorAll("details.menu[open]").forEach((menu) => {
          if (!menu.contains(event.target)) menu.open = false;
        });
      });
      document.addEventListener("keydown", (event) => {
        if (event.key === "Escape") {
          document.querySelectorAll("details.menu[open]").forEach((menu) => {
            menu.open = false;
            menu.querySelector("summary").focus();
          });
        }
      });
      document.querySelectorAll(".multi-select").forEach((menu) => this.multiSelect(menu));
      this.controls(document);
      document.addEventListener("click", (event) => {
        const button = event.target.closest("[data-copy-value]");
        if (button) this.copy(button);
      });
      window.addEventListener("resize", () => document.querySelectorAll(".fidelity-preview").forEach((preview) => this.resizePreview(preview)));
      const toastStack = document.createElement("div");
      toastStack.className = "toast-stack";
      toastStack.setAttribute("aria-label", "Notifications");
      document.body.append(toastStack);
      this.toastStack = toastStack;
    },
    copyField(label, value) {
      return '<div class="field"><span>' + this.escape(label) + '</span><div class="copy-field"><code tabindex="0">' + this.escape(value) + '</code>' +
        '<button class="btn ghost icon-only small" type="button" data-copy-value="' + this.escape(value) + '" aria-label="Copy ' + this.escape(label.toLowerCase()) + '">' + this.icon("copy") + '</button><span class="copy-message" role="status" aria-live="polite"></span></div></div>';
    },
    async copy(button) {
      if (button.getAttribute("aria-busy") === "true") return;
      const field = button.closest(".copy-field");
      const message = field.querySelector(".copy-message");
      clearTimeout(button.copyTimer);
      field.dataset.state = "copying";
      message.textContent = "";
      button.setAttribute("aria-busy", "true");
      button.innerHTML = '<span class="spinner" aria-hidden="true"></span>';
      let succeeded = false;
      try {
        if (!navigator.clipboard || !navigator.clipboard.writeText) throw new Error("Clipboard unavailable");
        await navigator.clipboard.writeText(button.dataset.copyValue);
        succeeded = true;
      } catch (error) {
        // The selectable value remains available when the host denies clipboard access.
        message.textContent = "Copy failed. Select and copy the value.";
      }
      button.removeAttribute("aria-busy");
      field.dataset.state = succeeded ? "copied" : "error";
      button.innerHTML = this.icon(succeeded ? "check" : "warning");
      if (succeeded) message.textContent = "Copied to clipboard.";
      button.copyTimer = setTimeout(() => {
        field.dataset.state = "";
        button.innerHTML = this.icon("copy");
        message.textContent = "";
      }, succeeded ? 1800 : 8000);
    },
    stepper(id, label, value, min, max, step, unit) {
      return '<div class="field"><label for="' + id + '">' + this.escape(label) + '</label><div class="numeric-stepper" data-stepper>' +
        '<button class="btn ghost icon-only small" type="button" data-decrement aria-label="Decrease ' + this.escape(label.toLowerCase()) + '">' + this.icon("minus") + '</button>' +
        '<input class="input mono" type="number" id="' + id + '" value="' + value + '" min="' + min + '" max="' + max + '" step="' + step + '" required aria-label="' + this.escape(label + (unit ? ' (' + unit + ')' : '')) + '">' +
        (unit ? '<span class="stepper-unit" aria-hidden="true">' + this.escape(unit) + '</span>' : '') +
        '<button class="btn ghost icon-only small" type="button" data-increment aria-label="Increase ' + this.escape(label.toLowerCase()) + '">' + this.icon("plus") + '</button></div></div>';
    },
    numberValue(value, min, max, step, fallback) {
      const number = value === "" || !Number.isFinite(Number(value)) ? fallback : Number(value);
      const precision = Math.max(...[min, step].map((item) => (String(item).split(".")[1] || "").length));
      const snapped = min + Math.round((Math.min(max, Math.max(min, number)) - min) / step) * step;
      return Number(Math.min(max, Math.max(min, snapped)).toFixed(precision));
    },
    controls(root) {
      root.querySelectorAll("[data-stepper]:not([data-bound])").forEach((group) => {
        group.dataset.bound = "true";
        const input = group.querySelector("input");
        const minus = group.querySelector("[data-decrement]");
        const plus = group.querySelector("[data-increment]");
        const min = Number(input.min), max = Number(input.max), step = Number(input.step);
        let previous = Number(input.value);
        const update = () => {
          const value = input.value === "" ? previous : Number(input.value);
          minus.disabled = input.disabled || value <= min;
          plus.disabled = input.disabled || value >= max;
          input.setAttribute("aria-invalid", String(!input.validity.valid));
        };
        const commit = () => {
          const value = this.numberValue(input.value, min, max, step, previous);
          const changed = input.value !== String(value);
          input.value = value;
          previous = value;
          update();
          return changed;
        };
        input.addEventListener("input", update);
        input.addEventListener("stepper-sync", () => { previous = Number(input.value); update(); });
        input.addEventListener("change", commit);
        input.addEventListener("blur", () => {
          if (commit()) input.dispatchEvent(new Event("change", { bubbles: true }));
        });
        [minus, plus].forEach((button, index) => button.addEventListener("click", () => {
          commit();
          input.value = this.numberValue(Number(input.value) + (index ? step : -step), min, max, step, previous);
          input.dispatchEvent(new Event("change", { bubbles: true }));
        }));
        update();
      });
      root.querySelectorAll(".fidelity-preview:not([data-bound])").forEach((preview) => {
        preview.dataset.bound = "true";
        const input = preview.querySelector("input");
        const zoom = () => {
          if (!input.validity.valid) return;
          preview.dataset.fit = "false";
          this.resizePreview(preview);
        };
        input.addEventListener("input", zoom);
        input.addEventListener("change", zoom);
        preview.querySelector("[data-preview-fit]").addEventListener("click", () => {
          preview.dataset.fit = "true";
          this.resizePreview(preview);
        });
        this.resizePreview(preview);
      });
    },
    preview(id) {
      return '<figure class="fidelity-preview" data-fit="true"><div class="preview-viewport" tabindex="0" role="region" aria-label="Sample artwork preview"><div class="preview-canvas"><img src="../_neuro/samples/interface-kit.svg" width="640" height="400" alt="Sample artwork: blue panels, a red circle and all four numbered corners" draggable="false"></div></div>' +
        '<figcaption>Sample artwork · 640 × 400</figcaption><div class="preview-tools"><button class="btn small" type="button" data-preview-fit aria-pressed="true">Fit</button>' + this.stepper(id, "Zoom", 100, 1, 200, 1, "%") + '</div></figure>';
    },
    resizePreview(preview) {
      const viewport = preview.querySelector(".preview-viewport");
      const canvas = preview.querySelector(".preview-canvas");
      const image = preview.querySelector("img");
      const input = preview.querySelector("input");
      const fit = preview.dataset.fit === "true";
      let scale = Number(input.value) / 100;
      preview.querySelector("[data-preview-fit]").setAttribute("aria-pressed", String(fit));
      if (fit) {
        canvas.style.width = "100%"; canvas.style.height = "100%";
        scale = Math.min(1, viewport.clientWidth / 640, viewport.clientHeight / 400);
        input.value = Math.max(1, Math.round(scale * 100));
        viewport.scrollTo(0, 0);
      } else {
        canvas.style.width = Math.max(viewport.clientWidth, 640 * Number(input.value) / 100) + "px";
        canvas.style.height = Math.max(viewport.clientHeight, 400 * Number(input.value) / 100) + "px";
      }
      image.style.width = 640 * scale + "px";
      image.style.height = 400 * scale + "px";
      // Reconcile boundary buttons without treating Fit as a manual zoom request.
      input.dispatchEvent(new Event("stepper-sync"));
    },
    multiSelect(menu) {
      const panel = menu.querySelector(".menu-panel");
      const summary = menu.querySelector("[data-selection-summary]");
      const update = () => {
        const count = menu.querySelectorAll('input[type="checkbox"]:checked').length;
        summary.textContent = count ? count + " selected" : "None selected";
      };
      const place = () => {
        if (!menu.open) return;
        const box = menu.querySelector("summary").getBoundingClientRect();
        const below = innerHeight - box.bottom - 18;
        const above = box.top - 18;
        const upwards = below < Math.min(240, panel.scrollHeight) && above > below;
        menu.dataset.placement = upwards ? "above" : "below";
        panel.style.maxHeight = Math.max(0, Math.min(240, upwards ? above : below)) + "px";
      };
      menu.addEventListener("change", update);
      menu.addEventListener("toggle", place);
      window.addEventListener("resize", place);
      document.addEventListener("scroll", place, true);
      update();
    },
    toast(message) {
      while (this.toastStack.children.length >= 3) this.toastStack.firstElementChild.remove();
      const toast = document.createElement("div");
      toast.className = "toast";
      toast.innerHTML = this.icon("check") + '<span role="status"></span><button class="btn ghost icon-only small" aria-label="Dismiss notification" type="button">' + this.icon("close") + '</button>';
      toast.querySelector("span").textContent = message;
      toast.querySelector("button").addEventListener("click", () => toast.remove());
      this.toastStack.append(toast);
    },
    dialog(options) {
      const trigger = document.activeElement;
      const previousOverflow = document.body.style.overflow;
      const dialog = document.createElement("dialog");
      dialog.setAttribute("aria-labelledby", "dialog-title");
      if (options.description) dialog.setAttribute("aria-describedby", "dialog-description");
      dialog.innerHTML = '<button class="btn ghost icon-only small dialog-close" type="button" aria-label="Close dialog">' + this.icon("close") + '</button>' +
        '<form id="dialog-form"><h2 id="dialog-title">' + this.escape(options.title) + '</h2>' +
        (options.description ? '<p id="dialog-description">' + this.escape(options.description) + '</p>' : '') +
        (options.content || "") + '<p id="dialog-error" class="error" role="alert"></p>' +
        '<div class="dialog-actions"><button class="btn" type="button" data-cancel>Cancel</button>' +
        '<button class="btn ' + (options.danger ? "destructive" : "primary") + '" type="submit">' + this.escape(options.confirm || "Apply") + '</button></div></form>';
      document.body.append(dialog);
      document.body.style.overflow = "hidden";
      dialog.querySelector(".dialog-close").onclick = () => dialog.close();
      dialog.querySelector("[data-cancel]").onclick = () => dialog.close();
      dialog.addEventListener("click", (event) => {
        const rectangle = dialog.getBoundingClientRect();
        if (event.target === dialog && (event.clientX < rectangle.left || event.clientX > rectangle.right || event.clientY < rectangle.top || event.clientY > rectangle.bottom)) dialog.close();
      });
      dialog.addEventListener("close", () => {
        document.body.style.overflow = previousOverflow;
        dialog.remove();
        const returnTarget = options.returnFocus ? document.querySelector(options.returnFocus) : trigger;
        if (returnTarget && returnTarget.isConnected) returnTarget.focus();
      }, { once: true });
      dialog.querySelector("form").addEventListener("submit", (event) => {
        event.preventDefault();
        const form = event.currentTarget;
        const error = dialog.querySelector("#dialog-error");
        const result = options.onConfirm ? options.onConfirm(new FormData(form), error) : true;
        if (result !== false) dialog.close();
      });
      dialog.showModal();
      if (options.danger) dialog.querySelector("[data-cancel]").focus();
      return dialog;
    },
    tabs(group) {
      const tabs = Array.from(group.querySelectorAll('[role="tab"]'));
      const select = (tab) => {
        tabs.forEach((item) => {
          const selected = item === tab;
          item.setAttribute("aria-selected", String(selected));
          item.tabIndex = selected ? 0 : -1;
          document.getElementById(item.getAttribute("aria-controls")).hidden = !selected;
        });
      };
      tabs.forEach((tab, index) => {
        tab.addEventListener("click", () => select(tab));
        tab.addEventListener("keydown", (event) => {
          let next;
          if (event.key === "ArrowRight") next = (index + 1) % tabs.length;
          if (event.key === "ArrowLeft") next = (index - 1 + tabs.length) % tabs.length;
          if (event.key === "Home") next = 0;
          if (event.key === "End") next = tabs.length - 1;
          if (next !== undefined) {
            event.preventDefault();
            select(tabs[next]);
            tabs[next].focus();
          }
        });
      });
    },
    thumbnail(hasPreview) {
      return '<div class="mini-preview">' + (hasPreview ? '<img src="../_neuro/samples/interface-kit.svg" width="640" height="400" alt="Sample artwork thumbnail">' : this.icon("file") + '<small>No preview</small>') + '</div>';
    }
  };
})();
