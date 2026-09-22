import { BrowserRouter, Routes, Route } from 'react-router-dom'
import LoginView from './views/LoginView'
import ProfileView from './views/ProfileView'
import RegisterView from './views/RegisterView'
import LandingPage from './views/LandingPage'
import CoursesView from './views/EstudentCourses/CoursesView';


export default function Router() {

    return (
        <BrowserRouter>
            <Routes>
              <Route path='/auth/login' element={<LoginView />} />
              <Route path='/profile' element={<ProfileView/>} />
              <Route path='/auth/register' element={<RegisterView />} />
              <Route path='/courses' element={<CoursesView />} />
              <Route path='/' element={<LandingPage />} />
            </Routes>
        </BrowserRouter>
    )
}
