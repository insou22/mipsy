// This is all well and good, but we do NOT want to be serving 4MB of assets to our users! Tailwind has an inbuilt feature for purging unused CSS classes, which it does by searching for the classes you've used. 
// It does that using a regular expression, which just analyses the file, meaning Rust is fine!

module.exports = {
    content: [
        "./src/**/*.rs",
        "./index.html",
    ],
    theme: {
        extend: {
            colors: {
                // page background
                'th-primary': '#fee2e2',
                // bg-color code/register/output areas
                'th-secondary': '#f0f0f0',
                // used for step highlighting
                'th-highlighting':'#34d399',
                // tab hover color
                'th-tabhover': '#fff0f0',
                // unselected tab
                'th-tabunselected': '#d19292',
            }
        }
    },
    plugins: [],
};
