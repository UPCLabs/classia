import { BrowserRouter, Routes, Route } from 'react-router-dom'
import LoginView from './views/LoginView'
import ProfileView from './views/ProfileView'
import RegisterView from './views/RegisterView'
import LandingPage from './views/LandingPage'
import CoursesView from './views/EstudentCourses/CoursesView';
import CourseCreateView from './views/EstudentCourses/CourseCreateView'
import CourseDetailView from './views/EstudentCourses/CourseDetailView'
import CourseEditView from './views/EstudentCourses/CourseEditView'
import ProtectedRoute from './auth/ProtectedRoute'


export default function Router() {

    return (
        <BrowserRouter>
            <Routes>
              <Route path='/auth/login' element={<LoginView />} />
              <Route path='/profile' element={<ProfileView/>} />
              <Route path='/auth/register' element={<RegisterView />} />
              <Route
                path="/courses"
                element={
                  <ProtectedRoute>
                    <CoursesView />
                  </ProtectedRoute>
                }
              />
              <Route
                path="/courses/new"
                element={
                  <ProtectedRoute allowedRoles={['Teacher', 'Admin', 'SuperAdmin']}>
                    <CourseCreateView />
                  </ProtectedRoute>
                }
              />
              <Route
                path="/courses/:courseId"
                element={
                  <ProtectedRoute>
                    <CourseDetailView />
                  </ProtectedRoute>
                }
              />
              <Route
                path="/courses/:courseId/edit"
                element={
                  <ProtectedRoute allowedRoles={['Teacher', 'Admin', 'SuperAdmin']}>
                    <CourseEditView />
                  </ProtectedRoute>
                }
              />
              <Route path='/' element={<LandingPage />} />
            </Routes>
        </BrowserRouter>
    )
}
