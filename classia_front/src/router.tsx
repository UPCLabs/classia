import { BrowserRouter, Routes, Route } from 'react-router-dom'
import LoginView from './views/LoginView'
import ProfileView from './views/ProfileView'


export default function Router() {

    return (
        <BrowserRouter>
            <Routes>
              <Route path='/auth/login' element={<LoginView />} />
              <Route path='/profile' element={<ProfileView/>} />
            </Routes>
        </BrowserRouter>
    )
}
