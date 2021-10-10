// This is all well and good, but we do NOT want to be serving 4MB of assets to our users! Tailwind has an inbuilt feature for purging unused CSS classes, which it does by searching for the classes you've used. 
// It does that using a regular expression, which just analyses the file, meaning Rust is fine!

module.exports = {
    purge: {
        mode: "all",
        content: [
            "./src/**/*.rs",
            "./index.html",
            "./src/**/*.html",
            "./src/**/*.css",
        ],
    },
    theme: {
        extend: {
            colors: {
                // page background
                'th-primary': '#fee2e2',
                // bg-color code/register/output areas
                'th-secondary': '#d1d5db',
                // used for step highlighting
                'th-highlighting':'#34d399',
                // selected tab
                'th-tabclicked': '#d19292',
            }
        }
    },
    variants: {},
    plugins: [],
};
