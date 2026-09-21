import { BrowserRouter, Routes, Route } from 'react-router-dom'
import LoginView from './views/LoginView'


export default function Router() {

    return (
        <BrowserRouter>
            <Routes>
              <Route path='/auth/login' element={<LoginView />} />
            </Routes>
        </BrowserRouter>
    )
}
