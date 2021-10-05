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
                // this  is the colour of the background
                'th-primary': '#fee2e2',
                // this is the colour of the code/register/output areas
                'th-secondary': '#d1d5db',
                // this is the colour used for step highlighting
                'th-highlighting':'#34d399',
            }
        }
    },
    variants: {},
    plugins: [],
};
