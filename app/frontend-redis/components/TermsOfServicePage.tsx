import React from 'react';

const TermSection: React.FC<{ title: string; children: React.ReactNode }> = ({ title, children }) => (
    <section className="mb-6">
        <h2 className="text-xl font-bold text-slate-800 mb-2">{title}</h2>
        <div className="text-slate-600 space-y-3">
            {children}
        </div>
    </section>
);


const TermsOfServicePage: React.FC = () => {
    return (
        <div className="bg-white p-6 sm:p-10 rounded-xl shadow-md border border-slate-200">
             <div className="mb-8 text-center">
                <h1 className="text-4xl font-extrabold text-slate-900">Terms of Service</h1>
                <p className="mt-2 text-lg text-slate-600">Last Updated: {new Date().toLocaleDateString()}</p>
            </div>

            <p className="mb-6 text-slate-700">Please read these Terms of Service ("Terms", "Terms of Service") carefully before using the Redis Cloud Dashboard (the "Service") operated by us. Your access to and use of the Service is conditioned on your acceptance of and compliance with these Terms. These Terms apply to all visitors, users, and others who access or use the Service.</p>

            <TermSection title="1. Accounts">
                <p>When you create an account with us, you must provide information that is accurate, complete, and current at all times. Failure to do so constitutes a breach of the Terms, which may result in immediate termination of your account on our Service.</p>
                <p>You are responsible for safeguarding the password that you use to access the Service and for any activities or actions under your password. You agree not to disclose your password to any third party. You must notify us immediately upon becoming aware of any breach of security or unauthorized use of your account.</p>
            </TermSection>

            <TermSection title="2. Service Provision and Use">
                <p>The Service allows you to create, manage, and delete organizations and Redis instances. You are solely responsible for the data you store and the applications you run using the Redis instances provisioned through this Service.</p>
                <p>You agree not to use the Service for any unlawful purpose or to conduct any unlawful activity, including, but not limited to, fraud, embezzlement, money laundering, or insider trading.</p>
            </TermSection>

             <TermSection title="3. Acceptable Use">
                <p>You agree not to misuse the Service or help anyone else to do so. This includes, but is not limited to, probing, scanning, or testing the vulnerability of any system or network; breaching or otherwise circumventing any security or authentication measures; accessing, tampering with, or using non-public areas of the Service; or interfering with or disrupting any user, host, or network.</p>
            </TermSection>

            <TermSection title="4. Termination">
                <p>We may terminate or suspend your account immediately, without prior notice or liability, for any reason whatsoever, including without limitation if you breach the Terms.</p>
                <p>Upon termination, your right to use the Service will immediately cease. If you wish to terminate your account, you may do so by deleting your organizations and discontinuing use of the Service.</p>
            </TermSection>

            <TermSection title="5. Disclaimer of Warranties">
                <p>The Service is provided on an "AS IS" and "AS AVAILABLE" basis. The Service is provided without warranties of any kind, whether express or implied, including, but not limited to, implied warranties of merchantability, fitness for a particular purpose, non-infringement, or course of performance.</p>
            </TermSection>

            <TermSection title="6. Limitation Of Liability">
                <p>In no event shall our company, nor its directors, employees, partners, agents, suppliers, or affiliates, be liable for any indirect, incidental, special, consequential or punitive damages, including without limitation, loss of profits, data, use, goodwill, or other intangible losses, resulting from your access to or use of or inability to access or use the Service.</p>
            </TermSection>

             <TermSection title="7. Changes">
                <p>We reserve the right, at our sole discretion, to modify or replace these Terms at any time. We will try to provide at least 30 days' notice prior to any new terms taking effect. What constitutes a material change will be determined at our sole discretion.</p>
            </TermSection>

            <TermSection title="8. Contact Us">
                <p>If you have any questions about these Terms, please contact us at support@example.com.</p>
            </TermSection>
        </div>
    );
};

export default TermsOfServicePage;