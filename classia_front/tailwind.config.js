/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'verde-oscuro': '#123F36',
        'verde-medio': '#2A6B5C',
        dorado: '#C49A45',
        crema: '#E8DCC4',
      },
    },
  },
  plugins: [],
}
