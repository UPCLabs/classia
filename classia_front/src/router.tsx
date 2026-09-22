import { BrowserRouter, Routes, Route } from 'react-router-dom'
import LoginView from './views/LoginView'
import ProfileView from './views/ProfileView'
import RegisterView from './views/RegisterView'
import LandingPage from './views/LandingPage'


export default function Router() {

    return (
        <BrowserRouter>
            <Routes>
              <Route path='/login' element={<LoginView />} />
              <Route path='/profile' element={<ProfileView/>} />
              <Route path='/register' element={<RegisterView />} />
              <Route path='/' element={<LandingPage />} />
            </Routes>
        </BrowserRouter>
    )
}
