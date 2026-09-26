import { BrowserRouter, Routes, Route } from 'react-router-dom'
import LoginView from './views/LoginView'
import ProfileView from './views/ProfileView'
import LandingPage from './views/LandingPage'
import RegisterView from './views/RegisterView'
import CoursesView from './views/StudentCourses/CoursesView'
import CourseCreateView from './views/StudentCourses/CourseCreateView'
import CourseDetailView from './views/StudentCourses/CourseDetailView'
import CourseEditView from './views/StudentCourses/CourseEditView'
import ChangePasswordView from './views/Account/ChangePasswordView'
import DashboardView from './views/DashboardView'
import { AuthProvider } from './auth/AuthProvider'
import ProtectedRoute from './auth/ProtectedRoute'
import PublicOnlyRoute from './auth/PublicOnlyRoute'
import AuthenticatedLayout from './layouts/AuthenticatedLayout'
import AdminUsersView from './views/AdminUsers/AdminUsersView'
import EditUserView from './views/AdminUsers/EditUserView'

export default function Router() {
  return (
    <BrowserRouter>
      <AuthProvider>
        <Routes>
          <Route
            path="/auth/login"
            element={
              <PublicOnlyRoute>
                <LoginView />
              </PublicOnlyRoute>
            }
          />
          <Route
            element={
              <ProtectedRoute>
                <AuthenticatedLayout />
              </ProtectedRoute>
            }
          >
            <Route
              path="/admin/users"
              element={
                <ProtectedRoute allowedRoles={['SuperAdmin', 'Admin']}>
                  <AdminUsersView />
                </ProtectedRoute>
              }
            />
            <Route
              path="/admin/users/new"
              element={
                <ProtectedRoute allowedRoles={['SuperAdmin', 'Admin']}>
                  <RegisterView />
                </ProtectedRoute>
              }
            />
            <Route
              path="/admin/users/:userId/edit"
              element={
                <ProtectedRoute allowedRoles={['SuperAdmin', 'Admin']}>
                  <EditUserView />
                </ProtectedRoute>
              }
            />
            <Route path="/dashboard" element={<DashboardView />} />
            <Route path="/profile" element={<ProfileView />} />
            <Route
              path="/account/change-password"
              element={<ChangePasswordView />}
            />
            <Route
              path="/change-password"
              element={<ChangePasswordView />}
            />
            <Route path="/courses" element={<CoursesView />} />
            <Route
              path="/courses/new"
              element={
                <ProtectedRoute
                  allowedRoles={['Teacher', 'Admin', 'SuperAdmin']}
                >
                  <CourseCreateView />
                </ProtectedRoute>
              }
            />
            <Route path="/courses/:courseId" element={<CourseDetailView />} />
            <Route
              path="/courses/:courseId/edit"
              element={
                <ProtectedRoute
                  allowedRoles={['Teacher', 'Admin', 'SuperAdmin']}
                >
                  <CourseEditView />
                </ProtectedRoute>
              }
            />
          </Route>
          <Route path="/" element={<LandingPage />} />
        </Routes>
      </AuthProvider>
    </BrowserRouter>
  )
}
