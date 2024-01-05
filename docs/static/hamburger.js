const currentPath = window.location.pathname;

const hamburger = document.querySelector(".hamburger");
const hamburgerBg = document.querySelector(".hamburger_bg");
const nav = document.getElementsByTagName("nav")[0];
const pages = currentPath.startsWith('/docs') ? document.querySelector(".pages_for_hamburger") : null;

hamburger.addEventListener('click', () => {
    hamburgerBg.style.display = "block";
    
    if (currentPath.startsWith('/docs')) {
        pages.style.display = "flex";
    } else {
        nav.style.display = "flex";
        nav.style.zIndex = 50;
    }
});

hamburgerBg.addEventListener('click', () => {
    hamburgerBg.style.display = "none";

    if (currentPath.startsWith("/docs")) {
        pages.style.display = "none";
    } else {
        nav.style.display = "none";
        nav.style.zIndex = 0;
    }
});
