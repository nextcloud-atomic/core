/** @type {import('tailwindcss').Config} */
// const plugin = require("tailwindcss/plugin");

module.exports = {
  mode: "all",
  content: ["./src/**/*.{rs,html,css}", "./components/*.ts", "./dist/**/*.html"],
  safelist: [
    {
      pattern: /avatar*|alert*|modal*|btn*|menu*|dropdown*|badge*|card*|input*|select*|textarea*|label*|tab*|tooltip*|flex*|text*|overflow*|collapse*|bg*|border*|list*|bg-primary\/*/
    }
  ],
  theme: {
    extend: {},
  },
    plugins: [
        require("daisyui"),
        require('@tailwindcss/typography')
    ],

    // daisyUI config (optional - here are the default values)
    // daisyui: {
    //     themes: [
    //         {
    //             "ncatomic-light": {
    //                 ...require("daisyui/src/theming/themes")["garden"],
    //                 "accent": "oklch(56.19% 0.2208 339.75)",
    //                 "accent-content": "oklch(100% 0 0)",
    //                 "secondary": "oklch(66.17% 0.1021 243.7)",
    //                 "secondary-content": "oklch(100% 0 0)",
    //                 "primary": "oklch(58.39% 0.1431 243.7)",
    //                 "primary-content": "oklch(100% 0 0)",
    //                 "info": "oklch(90% 0.058 230.902)",
    //                 "info-content": "oklch(0% 0 0)",
    //             }
    //         }
    //     ], // false: only light + dark | true: all themes | array: specific themes like this ["light", "dark", "cupcake"]
    //     darkTheme: "light", // name of one of the included themes for dark mode
    //     base: true, // applies background color and foreground color for root element by default
    //     styled: true, // include daisyUI colors and design decisions for all components
    //     utils: true, // adds responsive and modifier utility classes
    //     prefix: "", // prefix for daisyUI classnames (components, modifiers and responsive class names. Not colors)
    //     logs: true, // Shows info about daisyUI version and used config in the console when building your CSS
    //     themeRoot: ":root", // The element that receives theme color CSS variables
    // },
    // daisyui: {
    //   themes: [
    //
    //   ]
    // }
};
