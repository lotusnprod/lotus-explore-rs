(function() {
    // 1. Non-blocking Analytics Initialization
    var loadAnalytics = function() {
        if (document.querySelector('script[src*="simpleanalyticscdn.com"]')) return;
        var s = document.createElement('script');
        s.async = true;
        s.defer = true;
        s.src = 'https://scripts.simpleanalyticscdn.com/latest.js';
        document.head.appendChild(s);
    };

    if ('requestIdleCallback' in window) {
        requestIdleCallback(loadAnalytics, { timeout: 2000 });
    } else {
        setTimeout(loadAnalytics, 1500);
    }
})();