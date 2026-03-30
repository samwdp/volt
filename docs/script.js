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
        if (pageSidebar) {
            const isOpen = pageSidebar.classList.toggle("is-open");
            navToggle.setAttribute("aria-expanded", String(isOpen));
        } else if (topbar) {
            const isOpen = topbar.classList.toggle("is-open");
            navToggle.setAttribute("aria-expanded", String(isOpen));
        }
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

copyButtons.forEach((button) => {
    button.addEventListener("click", async () => {
        const text = button.dataset.copy;

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
    });
});
