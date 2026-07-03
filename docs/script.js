const topbar = document.querySelector(".topbar");
const navToggle = document.querySelector(".nav-toggle");
const pageSidebar = document.querySelector(".page-sidebar");
const navLinks = Array.from(document.querySelectorAll(".sidebar-nav a, .primary-nav a"));
const revealItems = document.querySelectorAll(".reveal");
const copyButtons = document.querySelectorAll(".copy-button");
const sections = navLinks
    .map((link) => {
        const href = link.getAttribute("href");
        return href && href.startsWith("#") ? document.querySelector(href) : null;
    })
    .filter(Boolean);

if (navToggle && (topbar || pageSidebar)) {
    navToggle.addEventListener("click", () => {
        let isOpen = false;
        if (topbar) {
            isOpen = topbar.classList.toggle("is-open");
        }
        if (pageSidebar) {
            pageSidebar.classList.toggle("is-open");
            if (!topbar) {
                isOpen = pageSidebar.classList.contains("is-open");
            }
        }
        navToggle.setAttribute("aria-expanded", String(isOpen));
    });

    navLinks.forEach((link) => {
        link.addEventListener("click", () => {
            if (pageSidebar) {
                pageSidebar.classList.remove("is-open");
            }
            if (topbar) {
                topbar.classList.remove("is-open");
            }
            navToggle.setAttribute("aria-expanded", "false");
        });
    });
}

if ("IntersectionObserver" in window) {
    const revealObserver = new IntersectionObserver(
        (entries) => {
            entries.forEach((entry) => {
                if (entry.isIntersecting) {
                    entry.target.classList.add("is-visible");
                    revealObserver.unobserve(entry.target);
                }
            });
        },
        {
            threshold: 0.16,
            rootMargin: "0px 0px -10% 0px",
        }
    );

    revealItems.forEach((item) => revealObserver.observe(item));

    const sectionObserver = new IntersectionObserver(
        (entries) => {
            entries.forEach((entry) => {
                if (!entry.isIntersecting) {
                    return;
                }

                navLinks.forEach((link) => {
                    const isActive = link.getAttribute("href") === `#${entry.target.id}`;
                    link.classList.toggle("is-active", isActive);
                });
            });
        },
        {
            threshold: 0.45,
            rootMargin: "-15% 0px -35% 0px",
        }
    );

    sections.forEach((section) => sectionObserver.observe(section));
} else {
    revealItems.forEach((item) => item.classList.add("is-visible"));
}

async function copyText(button, text) {
    if (!text) {
        return;
    }

    try {
        await navigator.clipboard.writeText(text);
        const original = button.textContent;
        button.textContent = "Copied";
        button.classList.add("is-copied");

        window.setTimeout(() => {
            button.textContent = original;
            button.classList.remove("is-copied");
        }, 1400);
    } catch (error) {
        button.textContent = "Copy failed";
        window.setTimeout(() => {
            button.textContent = "Copy";
        }, 1400);
    }
}

copyButtons.forEach((button) => {
    button.addEventListener("click", async () => {
        await copyText(button, button.dataset.copy);
    });
});

function initAutoCodeCopy() {
    document.querySelectorAll("pre > code").forEach((code) => {
        const pre = code.parentElement;
        if (!pre || pre.closest(".code-block")) {
            return;
        }

        const wrapper = document.createElement("div");
        wrapper.className = "code-block";
        pre.replaceWith(wrapper);
        wrapper.appendChild(pre);

        const button = document.createElement("button");
        button.type = "button";
        button.className = "copy-button";
        button.textContent = "Copy";
        button.addEventListener("click", async () => {
            await copyText(button, code.textContent);
        });
        wrapper.appendChild(button);
    });
}

function initTabs() {
    document.querySelectorAll("[data-tab-group]").forEach((group) => {
        const buttons = Array.from(group.querySelectorAll("[data-tab]"));
        const panels = Array.from(group.querySelectorAll("[data-tab-panel]"));

        function activate(tabId) {
            buttons.forEach((button) => {
                const isActive = button.dataset.tab === tabId;
                button.classList.toggle("is-active", isActive);
                button.setAttribute("aria-selected", String(isActive));
            });
            panels.forEach((panel) => {
                const isActive = panel.dataset.tabPanel === tabId;
                panel.classList.toggle("is-active", isActive);
                panel.hidden = !isActive;
            });
        }

        buttons.forEach((button) => {
            button.addEventListener("click", () => activate(button.dataset.tab));
        });

        const initial = buttons.find((button) => button.classList.contains("is-active")) || buttons[0];
        if (initial) {
            activate(initial.dataset.tab);
        }
    });
}

function initCollapsibles() {
    document.querySelectorAll("[data-collapse]").forEach((item) => {
        const trigger = item.querySelector("[data-collapse-trigger]");
        const body = item.querySelector("[data-collapse-body]");
        if (!trigger || !body) {
            return;
        }

        trigger.addEventListener("click", () => {
            const isOpen = item.classList.toggle("is-open");
            trigger.setAttribute("aria-expanded", String(isOpen));
            body.hidden = !isOpen;
        });
    });
}

function initConfigMap() {
    const map = document.querySelector("[data-config-map]");
    if (!map) {
        return;
    }

    const cards = Array.from(map.querySelectorAll("[data-config-target]"));
    const tabGroup = document.querySelector(map.dataset.configMap);
    if (!tabGroup || cards.length === 0) {
        return;
    }

    cards.forEach((card) => {
        card.addEventListener("click", () => {
            const tabId = card.dataset.configTarget;
            const button = tabGroup.querySelector(`[data-tab="${tabId}"]`);
            if (button) {
                button.click();
            }
            cards.forEach((entry) => {
                entry.classList.toggle("is-active", entry === card);
            });
            tabGroup.scrollIntoView({ behavior: "smooth", block: "nearest" });
        });
    });
}

function initDocSearch() {
    const input = document.querySelector("[data-doc-search]");
    if (!input) {
        return;
    }

    const targets = Array.from(document.querySelectorAll("[data-searchable]"));
    const hint = document.querySelector("[data-doc-search-hint]");

    input.addEventListener("input", () => {
        const query = input.value.trim().toLowerCase();
        let visibleCount = 0;

        targets.forEach((target) => {
            const haystack = target.textContent.toLowerCase();
            const matches = query.length === 0 || haystack.includes(query);
            target.classList.toggle("is-search-hidden", !matches);
            if (matches) {
                visibleCount += 1;
            }
        });

        if (hint) {
            if (query.length === 0) {
                hint.textContent = "Filter guides, hooks, and config keys.";
            } else if (visibleCount === 0) {
                hint.textContent = "No matches. Try workspace, oil, acp, or picker.";
            } else {
                hint.textContent = `${visibleCount} match${visibleCount === 1 ? "" : "es"} for “${input.value.trim()}”.`;
            }
        }
    });
}

initAutoCodeCopy();
initTabs();
initCollapsibles();
initConfigMap();
initDocSearch();
